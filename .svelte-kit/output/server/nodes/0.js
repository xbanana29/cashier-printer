

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.BUQqy06I.js","_app/immutable/chunks/Bzak7iHL.js","_app/immutable/chunks/5-eTZw8_.js","_app/immutable/chunks/CL0_CD3h.js","_app/immutable/chunks/C0INEqdn.js","_app/immutable/chunks/C_gWEq03.js","_app/immutable/chunks/Dz1Qw5Eh.js","_app/immutable/chunks/Bo7kAp5_.js","_app/immutable/chunks/Bb_lvwxT.js","_app/immutable/chunks/CjkQlsDr.js","_app/immutable/chunks/DH4dr_NT.js","_app/immutable/chunks/CVYSSf5Q.js"];
export const stylesheets = ["_app/immutable/assets/0.gmYr8YYm.css"];
export const fonts = [];
