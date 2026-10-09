#!/usr/bin/env node
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const changelogPath = path.join(__dirname, '..', 'CHANGELOG.md');

// 鍒嗙被鏄犲皠
const typeMap = {
  feat: '鏂板 | New',
  fix: '淇 | Fix',
  refactor: '鏀硅繘 | Improved',
  perf: '鏀硅繘 | Improved',
  rft: '鏀硅繘 | Improved',
  docs: '鏂囨。 | Docs',
  doc: '鏂囨。 | Docs',
  style: '鍏朵粬 | Other',
  build: '鍏朵粬 | Other',
  ci: '鑷姩鍖?| CI',
  test: '鍏朵粬 | Other',
  chore: '鍏朵粬 | Other',
};

// 涓枃鍏抽敭璇嶆槧灏?const chineseKeywords = {
  '鏂板': '鏂板 | New',
  '淇': '淇 | Fix',
  '鏇存柊': '鏀硅繘 | Improved',
  '鏀硅繘': '鏀硅繘 | Improved',
  '浼樺寲': '鏀硅繘 | Improved',
  '閲嶆瀯': '鏀硅繘 | Improved',
  '鏂囨。': '鏂囨。 | Docs',
};

// 蹇界暐鐨勫墠缂€
const IGNORE_PREFIXES = /^(?:build|style|debug)\s*(?:\([^)]*\))*:\s*/;

function parseArgs() {
  const args = process.argv.slice(2);
  let tagName = null;
  let latest = null;
  let withHash = false;
  let withCommitizen = false;
  let outputOnly = false;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--tag' || arg === '-t') {
      tagName = args[++i];
    } else if (arg === '--base' || arg === '--latest' || arg === '-b') {
      latest = args[++i];
    } else if (arg === '-wh' || arg === '--with-hash') {
      withHash = true;
    } else if (arg === '-wc' || arg === '--with-commitizen') {
      withCommitizen = true;
    } else if (arg === '--output-only') {
      outputOnly = true;
    } else if (arg === '--help' || arg === '-h') {
      console.log(`
鐢ㄦ硶: node generate-changelog.js [閫夐」]

閫夐」:
  --tag, -t <鏍囩>      鎸囧畾鍙戝竷鏍囩鍚嶇О
  --base, --latest, -b <鏍囩>  鎸囧畾鍩虹鏍囩
  -wh, --with-hash      鏄剧ず鎻愪氦鍝堝笇
  -wc, --with-commitizen  淇濈暀 commitizen 鍓嶇紑
  --output-only         浠呰緭鍑?changelog 鍐呭锛堢敤浜?CI/CD锛?  -h, --help            鏄剧ず甯姪淇℃伅
      `);
      process.exit(0);
    }
  }

  return { tagName, latest, withHash, withCommitizen, outputOnly };
}

function callCommand(command) {
  try {
    return execSync(command, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'pipe'] }).trim();
  } catch (e) {
    try {
      return execSync(command, { encoding: 'gbk', stdio: ['pipe', 'pipe', 'pipe'] }).trim();
    } catch {
      return '';
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
  // 妫€鏌ュ拷鐣ュ墠缂€
  if (IGNORE_PREFIXES.test(message)) {
    return null;
  }

  // 妫€鏌?commitizen 鍓嶇紑
  const m = message.match(/^(?<prefix>\w+)(?:\([\w\-]+\))?:\s*/);
  if (m) {
    const prefix = m.groups.prefix.toLowerCase();
    return typeMap[prefix] || '鍏朵粬 | Other';
  }

  // 妫€鏌ヤ腑鏂囧叧閿瘝
  for (const [keyword, category] of Object.entries(chineseKeywords)) {
    if (message.includes(keyword)) {
      return category;
    }
  }

  return '鍏朵粬 | Other';
}

function getCommits(latest = null) {
  let gitCommand;
  if (latest) {
    gitCommand = `git log ${latest}..HEAD --pretty=format:"%H%n%aN%n%s"`;
  } else {
    gitCommand = 'git log --pretty=format:"%H%n%aN%n%s" -n 50';
  }

  const output = callCommand(gitCommand);
  if (!output) return [];

  const commits = [];
  const lines = output.split('\n');

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
    '鏂板 | New': [],
    '淇 | Fix': [],
    '鏀硅繘 | Improved': [],
    '鏂囨。 | Docs': [],
    '鍏朵粬 | Other': [],
    '鑷姩鍖?| CI': [],
  };

  const contributors = new Set();

  for (const commit of commits) {
    if (commit.message.includes('[skip changelog]')) continue;

    // 蹇界暐 CI 鑷姩鎻愪氦鐨勭増鏈洿鏂版彁浜?    if (commit.message.match(/^chore:\s*update\s+version\s+to\s+.+/i)) continue;

    const category = parseCategory(commit.message);
    if (!category) continue;

    let message = commit.message;

    // 鍓ユ帀 commitizen 鍓嶇紑
    if (!withCommitizen) {
      message = message.replace(/^(?<prefix>\w+)(?:\([\w\-]+\))?:\s*/, '');
    }

    // 娣诲姞璐＄尞鑰?    if (commit.author && commit.author !== 'web-flow') {
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
    const remoteUrl = callCommand('git remote get-url origin');
    if (!remoteUrl) return null;

    // 杞崲 git@github.com:owner/repo.git 鎴?https://github.com/owner/repo.git 鏍煎紡
    let repoPath = remoteUrl;
    if (repoPath.startsWith('git@')) {
      repoPath = repoPath.replace('git@github.com:', 'https://github.com/');
    }
    repoPath = repoPath.replace('.git', '');
    return repoPath;
  } catch {
    return null;
  }
}

function generateMd(classifiedData, tagName, latest, withHash = false) {
  const { categories, contributors } = classifiedData;
  const now = new Date().toLocaleDateString('zh-CN');
  const lines = [];
  const repoUrl = getGitHubRepoUrl();

  // 鏍囬
  if (tagName) {
    lines.push(`## ${tagName}`);
  } else {
    lines.push(`## 馃摑 鏇存柊鏃ュ織 (${now})`);
  }

  if (latest && repoUrl) {
    const endTag = tagName || 'HEAD';
    lines.push(`> [${latest}...${endTag}](${repoUrl}/compare/${latest}...${endTag})`);
  } else if (latest) {
    lines.push(`> ${latest} ... HEAD`);
  }
  lines.push('');

  // 鎸夊垎绫婚『搴忚緭鍑?  const order = ['鏂板 | New', '淇 | Fix', '鏀硅繘 | Improved', '鏂囨。 | Docs', '鑷姩鍖?| CI', '鍏朵粬 | Other'];
  for (const category of order) {
    if (categories[category].length === 0) continue;

    lines.push(`### ${category}`);
    lines.push('');

    for (const item of categories[category]) {
      let line = `* ${item.message}`;
      /*if (withHash && repoUrl) {
        line += ` ([${item.hash}](${repoUrl}/commit/${item.hash}))`;
      } else if (withHash) {
        line += ` (${item.hash})`;
      }*/
      // 娣诲姞鎻愪氦鑰呬俊鎭?      if (item.author && item.author !== 'web-flow') {
        if (repoUrl) {
          line += ` @${item.author}`;
        } else {
          line += ` @${item.author}`;
        }
      }
      lines.push(line);
    }

    lines.push('');
  }

  return lines.join('\n');
}

function writeToFile(content, append = false) {
  if (append) {
    fs.appendFileSync(changelogPath, '\n' + content, 'utf8');
  } else {
    fs.writeFileSync(changelogPath, content, 'utf8');
  }
}

function main() {
  const { tagName, latest, withHash, withCommitizen, outputOnly } = parseArgs();

  const resolvedLatest = latest || getLatestTag();
  const resolvedTagName = tagName || getCurrentTag();

  if (!outputOnly) {
    console.log('馃搳 姝ｅ湪鐢熸垚鍙樻洿鏃ュ織...');
    if (resolvedLatest) {
      console.log(`馃搶 浠? ${resolvedLatest}`);
    }
    if (resolvedTagName) {
      console.log(`馃彿锔? 鍒? ${resolvedTagName}`);
    }
    console.log('');
  }

  const commits = getCommits(resolvedLatest);
  if (commits.length === 0) {
    if (!outputOnly) {
      console.log('鈿狅笍  娌℃湁鎵惧埌鎻愪氦璁板綍');
    }
    return;
  }

  const grouped = classifyCommits(commits, withCommitizen);
  const markdown = generateMd(grouped, resolvedTagName, resolvedLatest, withHash);

  writeToFile(markdown, !!resolvedLatest);

  if (outputOnly) {
    console.log(markdown);
  } else {
    console.log('鉁?鍙樻洿鏃ュ織宸叉洿鏂? CHANGELOG.md');
    console.log('\n' + markdown);
  }
}

main();