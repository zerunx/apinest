/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

interface Window {
  /** Called by MainView after the first workspace bootstrap (tab restore) finishes. */
  __NEST_MARK_SESSION_READY__?: () => void
}
