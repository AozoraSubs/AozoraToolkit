#!/usr/bin/env node
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

/**
 * 鎵ц鍛戒护骞惰繑鍥炵粨鏋滐紝闈欓粯澶勭悊閿欒
 */
function callCommand(command) {
  try {
    return execSync(command, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'pipe'] }).trim();
  } catch {
    return '';
  }
}

/**
 * 鑾峰彇褰撳墠骞翠唤鐨勫悗涓や綅
 */
function getYearSuffix() {
  return new Date().getFullYear().toString().slice(-2);
}

/**
 * 鑾峰彇褰撳墠鏄竴骞翠腑鐨勭鍑犲懆
 * ISO 鍛ㄦ暟锛氬懆涓€涓轰竴鍛ㄧ殑寮€濮嬶紝绗竴鍛ㄥ寘鍚骞寸殑绗竴涓懆鍥? */
function getWeekNumber() {
  const date = new Date();
  const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
  const dayNum = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  return Math.ceil((((d - yearStart) / 86400000) + 1) / 7);
}

/**
 * 鑾峰彇鎵€鏈夌幇鏈夌殑鍛ㄥ揩鐓ф爣绛? */
function getWeeklySnapshotTags() {
  const tags = callCommand('git tag -l').split('\n').filter(Boolean);
  const weeklyPattern = /^\d{2}w\d{1,2}[a-z]$/;
  return tags.filter(tag => weeklyPattern.test(tag)).sort();
}

/**
 * 鑾峰彇鎸囧畾鍛ㄧ殑鎵€鏈夊揩鐓ф爣绛? */
function getSnapshotsForWeek(yearSuffix, weekNum) {
  const tags = getWeeklySnapshotTags();
  const weekPrefix = `${yearSuffix}w${weekNum}`;
  return tags.filter(tag => tag.startsWith(weekPrefix));
}

/**
 * 鑾峰彇涓嬩竴涓増鏈瓧姣? */
function getNextLetter(existingTags, weekPrefix) {
  if (existingTags.length === 0) {
    return 'a';
  }

  const letters = existingTags.map(tag => {
    const match = tag.match(/^\d{2}w\d{1,2}([a-z])$/);
    return match ? match[1] : '';
  }).filter(Boolean).sort();

  const lastLetter = letters[letters.length - 1];
  if (!lastLetter) return 'a';

  const charCode = lastLetter.charCodeAt(0);
  if (charCode >= 122) {
    throw new Error('宸茶揪鍒版湰鍛ㄦ渶澶у揩鐓ф暟閲?(z)');
  }

  return String.fromCharCode(charCode + 1);
}

/**
 * 鐢熸垚涓嬩竴涓懆蹇収鐗堟湰鍙? */
function generateNextSnapshotVersion() {
  const yearSuffix = getYearSuffix();
  const weekNum = getWeekNumber();
  const weekPrefix = `${yearSuffix}w${weekNum}`;

  const existingTags = getSnapshotsForWeek(yearSuffix, weekNum);
  const nextLetter = getNextLetter(existingTags, weekPrefix);

  return `${weekPrefix}${nextLetter}`;
}

/**
 * 鑾峰彇褰撳墠鍛ㄥ揩鐓х殑鍩虹鏍囩
 * 閫昏緫锛氳幏鍙栨渶鏂板彂甯冪殑浠讳綍绫诲瀷鐨?tag锛堟棤璁烘槸 Pre Release銆丷elease 杩樻槸姣忓懆蹇収锛? */
function getBaseTag() {
  const allTags = callCommand('git tag -l --sort=-creatordate').split('\n').filter(Boolean);

  if (allTags.length > 0) {
    return allTags[0];
  }

  return null;
}

/**
 * 涓诲嚱鏁? */
function main() {
  try {
    if (process.argv.includes('--get-base-tag')) {
      const baseTag = getBaseTag();
      console.log(baseTag || '');
      return;
    }

    const nextVersion = generateNextSnapshotVersion();
    const baseTag = getBaseTag();

    console.log(nextVersion);

    if (process.argv.includes('--verbose')) {
      const yearSuffix = getYearSuffix();
      const weekNum = getWeekNumber();
      console.error(`骞翠唤鍚庝袱浣? ${yearSuffix}`);
      console.error(`鍛ㄦ暟: ${weekNum}`);
      console.error(`鍩虹鏍囩: ${baseTag || '鏃?}`);
      console.error(`涓嬩竴涓揩鐓х増鏈? ${nextVersion}`);
    }
  } catch (error) {
    console.error('鐢熸垚鍛ㄥ揩鐓х増鏈彿澶辫触:', error.message);
    process.exit(1);
  }
}

export {
  getYearSuffix,
  getWeekNumber,
  getWeeklySnapshotTags,
  getSnapshotsForWeek,
  getNextLetter,
  generateNextSnapshotVersion,
  getBaseTag,
};

const isDirectRun = process.argv[1] === fileURLToPath(import.meta.url);
if (isDirectRun) {
  main();
}
