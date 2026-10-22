import resolve from '@rollup/plugin-node-resolve';

export default {
  input: 'index.js',
  output: {
    file: '../static/js/vendor/MaterialWeb.js',
    format: 'es',
    sourcemap: false
  },
  plugins: [resolve()]
};
