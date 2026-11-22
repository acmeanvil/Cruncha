// SPDX-License-Identifier: GPL-3.0-or-later
// Vite's `?url` imports: the asset's URL in the built app.
declare module "*?url" {
  const url: string;
  export default url;
}
