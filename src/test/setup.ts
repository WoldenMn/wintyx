import "@testing-library/jest-dom/vitest";

const jsdom = (globalThis as { jsdom?: { window: Window } }).jsdom;

if (jsdom) {
  for (const cle of ["localStorage", "sessionStorage"] as const) {
    Object.defineProperty(globalThis, cle, {
      value: jsdom.window[cle],
      configurable: true,
      writable: true,
    });
  }
}
