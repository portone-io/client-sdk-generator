declare global {
  interface Window {
    PortOne: PortOne | undefined;
  }
}

let promise: Promise<PortOne> | null = null;
const portone = {
  jsSdkUrl:
    process.env.JS_SDK_URL ?? "https://cdn.portone.io/v2/browser-sdk.js",
};
function findScript(): HTMLScriptElement | null {
  return document.querySelector<HTMLScriptElement>(
    `script[src="${portone.jsSdkUrl}"]`
  );
}
function injectScript(): HTMLScriptElement {
  const script = document.createElement("script");
  script.src = portone.jsSdkUrl;
  const headOrBody = document.head || document.body;
  if (!headOrBody)
    throw new Error("[PortOne] Expected document.body not to be null");
  return headOrBody.appendChild(script);
}
export function loadScript() {
  if (promise != null) {
    // SDK is already loaded
    return promise;
  }
  return (promise = new Promise((resolve, reject) => {
    if (window.PortOne) {
      // window.PortOne is already injected by CDN
      return resolve(window.PortOne);
    }
    try {
      // window.PortOne will be injected by @portone/browser-sdk
      const script = findScript() || injectScript();
      script.addEventListener("load", () => {
        // script has just loaded
        if (window.PortOne) {
          // window.PortOne is successfully injected @portone/browser-sdk
          return resolve(window.PortOne);
        }
        reject(new Error("[PortOne] Failed to load window.PortOne"));
      });
      script.addEventListener("error", () => {
        reject(new Error("[PortOne] Failed to load window.PortOne"));
      });
    } catch (error) {
      return reject(error);
    }
  }));
}
export function setPortOneJsSdkUrl(url: string) {
  return (portone.jsSdkUrl = url);
}
