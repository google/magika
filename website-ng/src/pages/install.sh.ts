import type { APIContext } from "astro";
// We don't want a <meta> redirection page, so we disable prerendering.
export const prerender = false;

export async function GET({ redirect }: APIContext) {
  console.log("Redirecting to latest installer script");
  return redirect(
    "https://github.com/google/magika/releases/download/cli-latest/magika-installer.sh",
    302
  );
}
