export type PageId = "encode" | "jobs" | "presets" | "settings";

/** 全局导航状态：当前激活的页面，跨组件共享。 */
export const navigation = $state<{ activePage: PageId }>({
  activePage: "encode",
});
