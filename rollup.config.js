import typescript from '@rollup/plugin-typescript';
import { defineConfig } from 'rollup';

export default defineConfig({
  input: 'guest-js/index.ts',
  output: [
    {
      file: 'dist-js/index.js',
      format: 'esm',
    },
    {
      file: 'dist-js/index.cjs',
      format: 'cjs',
    },
  ],
  plugins: [
    typescript({
      declaration: true,
      declarationDir: 'dist-js',
      rootDir: 'guest-js',
    }),
  ],
  external: [/^@tauri-apps\/api/],
});
