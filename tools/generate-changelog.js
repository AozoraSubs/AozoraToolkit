#!/usr/bin/env node
import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const changelogPath = path.join(__dirname, "..", "CHANGELOG.md");

// 分类映射
const typeMap = {
  feat: "新增 | New",
  fix: "修复 | Fix",
  refactor: "改进 | Improved",
  perf: "改进 | Improved",
  rft: "改进 | Improved",
  docs: "文档 | Docs",
  doc: "文档 | Docs",
  style: "其他 | Other",
  build: "其他 | Other",
  ci: "自动化 | CI",
  test: "其他 | Other",
  chore: "其他 | Other",
};

// 中文关键词映射
const chineseKeywords = {
  新增: "新增 | New",
  修复: "修复 | Fix",
  更新: "改进 | Improved",
  改进: "改进 | Improved",
  优化: "改进 | Improved",
  重构: "改进 | Improved",
  文档: "文档 | Docs",
};

// 忽略的前缀
const IGNORE_PREFIXES = /^(?:build|style|debug)\s*(?:\([^)]*\))*:\s*/;

function parseArgs() {
  const args = process.argv.slice(2);
  let tagName = null;
  let latest = null;
  let all = false;
  let withHash = false;
  let withCommitizen = false;
  let outputOnly = false;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--tag" || arg === "-t") {
      tagName = args[++i];
    } else if (arg === "--base" || arg === "--latest" || arg === "-b") {
      latest = args[++i];
    } else if (arg === "--all" || arg === "-a") {
      all = true;
    } else if (arg === "-wh" || arg === "--with-hash") {
      withHash = true;
    } else if (arg === "-wc" || arg === "--with-commitizen") {
      withCommitizen = true;
    } else if (arg === "--output-only") {
      outputOnly = true;
    } else if (arg === "--help" || arg === "-h") {
      console.log(`
用法: node generate-changelog.js [选项]

选项:
  --tag, -t <标签>      指定发布标签名称
  --base, --latest, -b <标签>  指定基础标签（上一个正式 Release；发布流程必传）
  --all, -a             从最初提交开始（仓库还没有正式 Release 时使用）
  -wh, --with-hash      显示提交哈希
  -wc, --with-commitizen  保留 commitizen 前缀
  --output-only         仅输出 changelog 内容（用于 CI/CD）
  -h, --help            显示帮助信息
      `);
      process.exit(0);
    }
  }

  return { tagName, latest, all, withHash, withCommitizen, outputOnly };
}

function callCommand(command) {
  try {
    return execSync(command, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    }).trim();
  } catch {
    try {
      return execSync(command, {
        encoding: "gbk",
        stdio: ["pipe", "pipe", "pipe"],
      }).trim();
    } catch {
      return "";
    }
  }
}

function getLatestTag() {
  try {
    return callCommand('git describe --tags --match "v*" --abbrev=0');
  } catch {
    return null;
  }
}

function getCurrentTag() {
  try {
    return callCommand('git describe --tags --match "v*"');
  } catch {
    return null;
  }
}

function parseCategory(message) {
  // 检查忽略前缀
  if (IGNORE_PREFIXES.test(message)) {
    return null;
  }

  // 检查 commitizen 前缀
  const m = message.match(/^(?<prefix>\w+)(?:\([\w-]+\))?:\s*/);
  if (m) {
    const prefix = m.groups.prefix.toLowerCase();
    return typeMap[prefix] || "其他 | Other";
  }

  // 检查中文关键词
  for (const [keyword, category] of Object.entries(chineseKeywords)) {
    if (message.includes(keyword)) {
      return category;
    }
  }

  return "其他 | Other";
}

function getCommits(latest = null, fromBeginning = false) {
  let gitCommand;
  if (fromBeginning) {
    // 明确要求全部历史（首次正式发布）
    gitCommand = 'git log HEAD --pretty=format:"%H%n%aN%n%s"';
  } else if (latest) {
    gitCommand = `git log ${latest}..HEAD --pretty=format:"%H%n%aN%n%s"`;
  } else {
    // 本地预览兜底：只取最近 50 条；发布流程必须显式传 --base 或 --all
    gitCommand = 'git log --pretty=format:"%H%n%aN%n%s" -n 50';
  }

  const output = callCommand(gitCommand);
  if (!output) return [];

  const commits = [];
  const lines = output.split("\n");

  for (let i = 0; i < lines.length; i += 3) {
    if (i + 2 >= lines.length) break;
    commits.push({
      hash: lines[i],
      author: lines[i + 1],
      message: lines[i + 2],
    });
  }

  return commits;
}

function classifyCommits(commits, withCommitizen = false) {
  const result = {
    "新增 | New": [],
    "修复 | Fix": [],
    "改进 | Improved": [],
    "文档 | Docs": [],
    "其他 | Other": [],
    "自动化 | CI": [],
  };

  const contributors = new Set();

  for (const commit of commits) {
    if (commit.message.includes("[skip changelog]")) continue;

    // 忽略 CI 自动提交的版本更新提交
    if (commit.message.match(/^chore:\s*update\s+version\s+to\s+.+/i)) continue;

    const category = parseCategory(commit.message);
    if (!category) continue;

    let message = commit.message;

    // 剥掉 commitizen 前缀
    if (!withCommitizen) {
      message = message.replace(/^(?<prefix>\w+)(?:\([\w-]+\))?:\s*/, "");
    }

    // 添加贡献者
    if (commit.author && commit.author !== "web-flow") {
      contributors.add(commit.author);
    }

    result[category].push({
      message,
      author: commit.author,
      hash: commit.hash.slice(0, 8),
    });
  }

  return { categories: result, contributors: Array.from(contributors) };
}

function getGitHubRepoUrl() {
  try {
    const remoteUrl = callCommand("git remote get-url origin");
    if (!remoteUrl) return null;

    // 转换 git@github.com:owner/repo.git 或 https://github.com/owner/repo.git 格式
    let repoPath = remoteUrl;
    if (repoPath.startsWith("git@")) {
      repoPath = repoPath.replace("git@github.com:", "https://github.com/");
    }
    repoPath = repoPath.replace(".git", "");
    return repoPath;
  } catch {
    return null;
  }
}

function generateMd(classifiedData, tagName, latest, _withHash = false) {
  const { categories } = classifiedData;
  const now = new Date().toLocaleDateString("zh-CN");
  const lines = [];
  const repoUrl = getGitHubRepoUrl();

  // 标题
  if (tagName) {
    lines.push(`## ${tagName}`);
  } else {
    lines.push(`## 📝 更新日志 (${now})`);
  }

  if (latest && repoUrl) {
    const endTag = tagName || "HEAD";
    lines.push(
      `> [${latest}...${endTag}](${repoUrl}/compare/${latest}...${endTag})`,
    );
  } else if (latest) {
    lines.push(`> ${latest} ... HEAD`);
  }
  lines.push("");

  // 按分类顺序输出
  const order = [
    "新增 | New",
    "修复 | Fix",
    "改进 | Improved",
    "文档 | Docs",
    "自动化 | CI",
    "其他 | Other",
  ];
  for (const category of order) {
    if (categories[category].length === 0) continue;

    lines.push(`### ${category}`);
    lines.push("");

    for (const item of categories[category]) {
      let line = `* ${item.message}`;
      /*if (withHash && repoUrl) {
        line += ` ([${item.hash}](${repoUrl}/commit/${item.hash}))`;
      } else if (withHash) {
        line += ` (${item.hash})`;
      }*/
      // 添加提交者信息
      if (item.author && item.author !== "web-flow") {
        if (repoUrl) {
          line += ` @${item.author}`;
        } else {
          line += ` @${item.author}`;
        }
      }
      lines.push(line);
    }

    lines.push("");
  }

  return lines.join("\n");
}

function writeToFile(content) {
  // 始终追加：CHANGELOG.md 的历史段落绝不被覆盖（无 --base 时旧代码会整篇覆盖）
  fs.appendFileSync(changelogPath, "\n" + content, "utf8");
}

function main() {
  const { tagName, latest, all, withHash, withCommitizen, outputOnly } =
    parseArgs();

  // 基准：CI 只能传 --base（上一个正式 Release，releases/latest 天然排除 prerelease 快照）
  // 或 --all（仓库还没有正式 Release）。两者都没有时只做本地预览，不用于发布。
  const fromBeginning = !latest && all;
  const resolvedLatest = latest || (fromBeginning ? null : getLatestTag());
  const resolvedTagName = tagName || getCurrentTag();
  const target = resolvedTagName || "HEAD";

  // 范围信息一律写 stderr：--output-only 的 stdout 会被 CI 直接当作发布正文
  if (resolvedLatest) {
    console.error(`📌 变更范围: ${resolvedLatest}..${target}`);
  } else if (fromBeginning) {
    console.error(`📌 变更范围: 最初提交..${target}（--all）`);
  } else {
    console.error(
      "⚠️  未指定 --base 或 --all：仅统计最近 50 条提交（本地预览用途，不要用于发布）",
    );
  }

  const commits = getCommits(resolvedLatest, fromBeginning);
  if (commits.length === 0) {
    console.error("⚠️  没有找到提交记录");
    return;
  }

  const grouped = classifyCommits(commits, withCommitizen);
  const markdown = generateMd(
    grouped,
    resolvedTagName,
    resolvedLatest,
    withHash,
  );

  writeToFile(markdown);

  if (outputOnly) {
    console.log(markdown);
  } else {
    console.error("✅ 变更日志已追加到: CHANGELOG.md");
    console.log("\n" + markdown);
  }
}

main();
