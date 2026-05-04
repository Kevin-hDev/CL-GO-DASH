import type { McpConnectorSpec } from "@/types/mcp";

export const MCP_CATALOG_LOCAL: McpConnectorSpec[] = [
  {
    id: "context7", display_name: "Context7", category: "devtools", auth_type: "none",
    short_description: "Documentation et exemples de code à jour pour 1000+ librairies.",
    short_description_en: "Up-to-date docs and code examples for 1000+ libraries.",
    author: "Upstash", url: "https://context7.com",
    install_command: "npx @upstash/context7-mcp@2.2.3",
    tools: ["resolve_library_id", "query_docs"],
  },
  {
    id: "huggingface", display_name: "Hugging Face", category: "ai-ml", auth_type: "token",
    short_description: "Explorer les modèles, datasets, Spaces et papers du Hub.",
    short_description_en: "Explore Hub models, datasets, Spaces and papers.",
    author: "Hugging Face", url: "https://huggingface.co/mcp",
    install_command: "npx @llmindset/hf-mcp-server@0.3.11",
    env_keys: ["HF_TOKEN"],
    tools: ["model_search", "dataset_search", "spaces_search", "papers_search", "docs_search"],
  },
  {
    id: "imessage", display_name: "iMessage", category: "communication", auth_type: "none",
    short_description: "Accès lecture-seule aux conversations iMessage.",
    short_description_en: "Read-only access to iMessage conversations.",
    author: "Wyatt Johnson", url: "https://github.com/wyattjoh/imessage-mcp",
    install_command: "deno run --allow-read --allow-write --allow-env --allow-sys --allow-ffi --allow-net jsr:@wyattjoh/imessage-mcp",
    os_restrict: "macos",
    tools: ["search_messages", "get_recent_messages", "get_chats", "get_messages_from_chat"],
  },
  {
    id: "producthunt", display_name: "Product Hunt", category: "community", auth_type: "token",
    short_description: "Accès aux posts, collections et topics Product Hunt.",
    short_description_en: "Access Product Hunt posts, collections and topics.",
    author: "Jai Pandya", url: "https://www.producthunt.com",
    install_command: "uvx product-hunt-mcp",
    env_keys: ["PRODUCT_HUNT_TOKEN"],
    tools: ["get_posts", "get_post_details", "get_collections", "search_topics", "get_user"],
  },
  {
    id: "reddit", display_name: "Reddit", category: "community", auth_type: "none",
    short_description: "Lecture et écriture Reddit : posts, commentaires, trending.",
    short_description_en: "Read and write Reddit: posts, comments, trending.",
    author: "Jordan Burke", url: "https://www.reddit.com",
    install_command: "npx reddit-mcp-server@1.2.1",
    tools: ["read_posts", "search", "trending", "create_post", "reply", "edit"],
  },
];
