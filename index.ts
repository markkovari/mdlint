import { parseMarkdown } from "https://deno.land/x/markdown_wasm/mod.ts";

type Config = {
  path: string;
};

const config: Config = {
  path: Deno.args[0] || ".",
};

// Buckle up, this is a recursive function take a sip of coffee bruv
async function getMarkdownPaths(config: Config): Promise<string[]> {
  const files = Deno.readDir(config.path);
  const markdowns: string[] = [];
  for await (const file of files) {
    if (file.isDirectory) {
      const subMarkdowns = await getMarkdownPaths({
        path: `${config.path}/${file.name}`,
      });
      markdowns.push(...subMarkdowns);
    } else if (file.name.endsWith(".md") || file.name.endsWith(".markdown")) {
      markdowns.push(`${config.path}/${file.name}`);
    }
  }
  return markdowns;
}

async function readMarkdownFiles(paths: string[]): Promise<Uint8Array[]> {
  return Promise.all(
    paths.map((path) => Deno.readFile(parseMarkdown(path))),
  );
}
const markdowns = await getMarkdownPaths(config);
const parsedMarkdowns = await readMarkdownFiles(markdowns);

console.log({ markdowns, parsedMarkdowns });
