// Miniquad plugin: localStorage without wasm-bindgen (required for mq_js_bundle.js).
// See https://macroquad.rs/articles/wasm/
register_plugin = function (importObject) {
  const KEY = "flappy_rust_high_score";
  importObject.env = importObject.env || {};
  importObject.env.flappy_storage_load = function () {
    try {
      const v = localStorage.getItem(KEY);
      if (v === null) return -1;
      const n = parseInt(v, 10);
      if (!Number.isFinite(n) || n < 0) return -1;
      if (n > 2147483647) return 2147483647;
      return n | 0;
    } catch (_) {
      return -1;
    }
  };
  importObject.env.flappy_storage_save = function (score) {
    try {
      const u = score >>> 0;
      localStorage.setItem(KEY, String(u));
    } catch (_) {}
  };
};
miniquad_add_plugin({ register_plugin });
