import { describe, test, expect } from "vitest";
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkDirective from "remark-directive";
import remarkRehype from "remark-rehype";
import rehypeStringify from "rehype-stringify";
import type { Root as MdastRoot, Nodes as MdastNodes } from "mdast";
import { markdownToHtml, defineMdastPlugin } from "../../src/index.js";
import type { MdastPluginInstance } from "../../src/mdast/mdast-visitor.js";
import type { MdastNode } from "../../src/types.js";

// Each test exercises an mdast plugin that mirrors the canonical remark idiom
// of mutating `node.data.hName`/`hProperties`/`hChildren`. We run the same
// transform through both pipelines and compare the resulting HTML so satteri
// stays observably identical to remark-rehype's `applyData` semantics.

type MdastPluginFactory = () => MdastPluginInstance;

interface RemarkPluginAndSatteri {
  /** Mutates the mdast tree in place — the remark idiom. */
  remark: (tree: MdastRoot) => void;
  /** Equivalent satteri plugin shape. */
  satteri: MdastPluginFactory;
}

function visitMdast(tree: MdastNodes, fn: (node: MdastNodes) => void): void {
  fn(tree);
  if ("children" in tree && Array.isArray(tree.children)) {
    for (const child of tree.children as MdastNodes[]) {
      visitMdast(child, fn);
    }
  }
}

function referenceHtml(md: string, plugin: RemarkPluginAndSatteri["remark"]): string {
  const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkDirective)
    .use(() => (tree: MdastRoot) => plugin(tree))
    .use(remarkRehype, { allowDangerousHtml: true })
    .use(rehypeStringify, { allowDangerousHtml: true });
  return normalize(String(processor.processSync(md)));
}

function satteriHtml(md: string, plugin: MdastPluginFactory): string {
  const result = markdownToHtml(md, {
    features: { directive: true, gfm: true, frontmatter: false, math: false },
    mdastPlugins: [defineMdastPlugin({ name: "hdata-test", ...plugin() })],
  });
  if (typeof result !== "string") throw new Error("expected sync result");
  return normalize(result);
}

function normalize(html: string): string {
  return html
    .replace(/<br>/g, "<br />")
    .replace(/<br\/>/g, "<br />")
    .replace(/<hr>/g, "<hr />")
    .replace(/<hr\/>/g, "<hr />")
    .trim();
}

function assertHtmlMatches(md: string, plugin: RemarkPluginAndSatteri): void {
  const ref = referenceHtml(md, plugin.remark);
  const got = satteriHtml(md, plugin.satteri);
  expect(got).toBe(ref);
}

// Helpers that do the same thing on both sides for the common case where the
// mdast plugin only writes data fields.

interface DataPatch {
  hName?: string;
  hProperties?: Record<string, unknown>;
  hChildren?: unknown[];
}

function mutateOnRemark(
  predicate: (node: MdastNodes) => boolean,
  patch: DataPatch,
): RemarkPluginAndSatteri["remark"] {
  return (tree) => {
    visitMdast(tree, (node) => {
      if (predicate(node)) {
        const data = ((node as { data?: Record<string, unknown> }).data ??= {});
        if (patch.hName !== undefined) data.hName = patch.hName;
        if (patch.hProperties !== undefined) data.hProperties = patch.hProperties;
        if (patch.hChildren !== undefined) data.hChildren = patch.hChildren;
      }
    });
  };
}

function mutateOnSatteri(
  type: keyof MdastPluginInstance,
  predicate: (node: MdastNode) => boolean,
  patch: DataPatch,
): MdastPluginFactory {
  return () => ({
    [type]: ((node: MdastNode, ctx: { setProperty: Function }) => {
      if (!predicate(node)) return;
      const existing = ((node as { data?: Record<string, unknown> }).data ?? {}) as Record<
        string,
        unknown
      >;
      const next = { ...existing };
      if (patch.hName !== undefined) next.hName = patch.hName;
      if (patch.hProperties !== undefined) next.hProperties = patch.hProperties;
      if (patch.hChildren !== undefined) next.hChildren = patch.hChildren;
      ctx.setProperty(node, "data", next);
    }) as MdastPluginInstance[typeof type],
  });
}

describe("data.hName / hProperties / hChildren conformance vs remark-rehype", () => {
  test("hName on paragraph swaps the tag, keeps children", () => {
    assertHtmlMatches("Hello world", {
      remark: mutateOnRemark((n) => n.type === "paragraph", { hName: "section" }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", { hName: "section" }),
    });
  });

  test("hName on heading swaps h1 with div", () => {
    assertHtmlMatches("# Title\n\nbody", {
      remark: mutateOnRemark((n) => n.type === "heading", { hName: "div" }),
      satteri: mutateOnSatteri("heading", (n) => n.type === "heading", { hName: "div" }),
    });
  });

  test("hProperties merges onto paragraph defaults", () => {
    assertHtmlMatches("Hi", {
      remark: mutateOnRemark((n) => n.type === "paragraph", {
        hProperties: { className: ["note", "boxed"], id: "intro" },
      }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", {
        hProperties: { className: ["note", "boxed"], id: "intro" },
      }),
    });
  });

  test("hName + hProperties together", () => {
    assertHtmlMatches("Body", {
      remark: mutateOnRemark((n) => n.type === "paragraph", {
        hName: "aside",
        hProperties: { className: ["note"] },
      }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", {
        hName: "aside",
        hProperties: { className: ["note"] },
      }),
    });
  });

  test("hChildren replaces the rendered children", () => {
    assertHtmlMatches("original", {
      remark: mutateOnRemark((n) => n.type === "paragraph", {
        hChildren: [{ type: "text", value: "replaced" }],
      }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", {
        hChildren: [{ type: "text", value: "replaced" }],
      }),
    });
  });

  test("hName + hChildren emits an arbitrary subtree", () => {
    const tree = [
      {
        type: "element",
        tagName: "strong",
        properties: {},
        children: [{ type: "text", value: "Hi" }],
      },
    ];
    assertHtmlMatches("Original body", {
      remark: mutateOnRemark((n) => n.type === "paragraph", {
        hName: "aside",
        hProperties: { className: ["note"] },
        hChildren: tree,
      }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", {
        hName: "aside",
        hProperties: { className: ["note"] },
        hChildren: tree,
      }),
    });
  });

  test("hProperties on heading", () => {
    assertHtmlMatches("# Title", {
      remark: mutateOnRemark((n) => n.type === "heading", {
        hProperties: { id: "main", className: ["big"] },
      }),
      satteri: mutateOnSatteri("heading", (n) => n.type === "heading", {
        hProperties: { id: "main", className: ["big"] },
      }),
    });
  });

  test("hName on listItem keeps content", () => {
    assertHtmlMatches("- one\n- two\n", {
      remark: mutateOnRemark((n) => n.type === "listItem", { hName: "div" }),
      satteri: mutateOnSatteri("listItem", (n) => n.type === "listItem", { hName: "div" }),
    });
  });

  test("hName on container directive (canonical use case)", () => {
    assertHtmlMatches(":::note\nContent here\n:::", {
      remark: mutateOnRemark(
        (n) => n.type === "containerDirective" && (n as { name?: string }).name === "note",
        { hName: "aside", hProperties: { className: ["note"] } },
      ),
      satteri: mutateOnSatteri(
        "containerDirective",
        (n) => n.type === "containerDirective" && (n as { name?: string }).name === "note",
        { hName: "aside", hProperties: { className: ["note"] } },
      ),
    });
  });

  test("hName on leaf directive", () => {
    assertHtmlMatches("::break", {
      remark: mutateOnRemark(
        (n) => n.type === "leafDirective" && (n as { name?: string }).name === "break",
        { hName: "hr" },
      ),
      satteri: mutateOnSatteri(
        "leafDirective",
        (n) => n.type === "leafDirective" && (n as { name?: string }).name === "break",
        { hName: "hr" },
      ),
    });
  });

  test("hName on emphasis", () => {
    assertHtmlMatches("This is *italic* text.", {
      remark: mutateOnRemark((n) => n.type === "emphasis", {
        hName: "i",
        hProperties: { className: ["em"] },
      }),
      satteri: mutateOnSatteri("emphasis", (n) => n.type === "emphasis", {
        hName: "i",
        hProperties: { className: ["em"] },
      }),
    });
  });

  test("hName on strong", () => {
    assertHtmlMatches("**hello**", {
      remark: mutateOnRemark((n) => n.type === "strong", {
        hName: "b",
      }),
      satteri: mutateOnSatteri("strong", (n) => n.type === "strong", {
        hName: "b",
      }),
    });
  });

  test("hName on link adds rel attribute", () => {
    assertHtmlMatches("[click](https://example.com)", {
      remark: mutateOnRemark((n) => n.type === "link", {
        hProperties: { rel: ["noopener"], target: "_blank" },
      }),
      satteri: mutateOnSatteri("link", (n) => n.type === "link", {
        hProperties: { rel: ["noopener"], target: "_blank" },
      }),
    });
  });

  test("hName on blockquote", () => {
    assertHtmlMatches("> quoted", {
      remark: mutateOnRemark((n) => n.type === "blockquote", { hName: "aside" }),
      satteri: mutateOnSatteri("blockquote", (n) => n.type === "blockquote", { hName: "aside" }),
    });
  });

  test("hName on thematicBreak (void)", () => {
    assertHtmlMatches("---\n", {
      remark: mutateOnRemark((n) => n.type === "thematicBreak", { hName: "hr" }),
      satteri: mutateOnSatteri("thematicBreak", (n) => n.type === "thematicBreak", {
        hName: "hr",
      }),
    });
  });

  test("hProperties null strips an existing override", () => {
    // First add then remove on a paragraph: end state should match the no-op
    // case — vanilla `<p>`.
    assertHtmlMatches("plain", {
      remark: mutateOnRemark((n) => n.type === "paragraph", {
        hProperties: { className: null as unknown as string[] },
      }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", {
        hProperties: { className: null as unknown as string[] },
      }),
    });
  });

  test("hChildren with empty array produces empty element", () => {
    assertHtmlMatches("body", {
      remark: mutateOnRemark((n) => n.type === "paragraph", {
        hName: "div",
        hChildren: [],
      }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", {
        hName: "div",
        hChildren: [],
      }),
    });
  });

  test("nested element in hChildren", () => {
    const tree = [
      {
        type: "element",
        tagName: "span",
        properties: { className: ["wrap"] },
        children: [
          { type: "text", value: "outer " },
          {
            type: "element",
            tagName: "em",
            properties: {},
            children: [{ type: "text", value: "inner" }],
          },
        ],
      },
    ];
    assertHtmlMatches("body", {
      remark: mutateOnRemark((n) => n.type === "paragraph", { hChildren: tree }),
      satteri: mutateOnSatteri("paragraph", (n) => n.type === "paragraph", { hChildren: tree }),
    });
  });
});
