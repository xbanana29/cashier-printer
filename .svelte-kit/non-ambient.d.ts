
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	type MatcherParam<M> = M extends (param : string) => param is (infer U extends string) ? U : string;

	export interface AppTypes {
		RouteId(): "/" | "/edit" | "/edit/[id]" | "/history" | "/new" | "/settings";
		RouteParams(): {
			"/edit/[id]": { id: string }
		};
		LayoutParams(): {
			"/": { id?: string };
			"/edit": { id?: string };
			"/edit/[id]": { id: string };
			"/history": Record<string, never>;
			"/new": Record<string, never>;
			"/settings": Record<string, never>
		};
		Pathname(): "/" | `/edit/${string}` & {} | "/history" | "/new" | "/settings";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): "/favicon.png" | "/svelte.svg" | "/tauri.svg" | "/vite.svg" | string & {};
	}
}