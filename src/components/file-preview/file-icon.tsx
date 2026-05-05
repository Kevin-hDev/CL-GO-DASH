import { Icon } from "@iconify/react";
import defaultFile from "@iconify/icons-vscode-icons/default-file.js";
import cIcon from "@iconify/icons-vscode-icons/file-type-c.js";
import cppIcon from "@iconify/icons-vscode-icons/file-type-cpp.js";
import cssIcon from "@iconify/icons-vscode-icons/file-type-css.js";
import goIcon from "@iconify/icons-vscode-icons/file-type-go-lightblue.js";
import htmlIcon from "@iconify/icons-vscode-icons/file-type-html.js";
import javaIcon from "@iconify/icons-vscode-icons/file-type-java.js";
import jsIcon from "@iconify/icons-vscode-icons/file-type-js.js";
import jsonIcon from "@iconify/icons-vscode-icons/file-type-json.js";
import markdownIcon from "@iconify/icons-vscode-icons/file-type-markdown.js";
import pythonIcon from "@iconify/icons-vscode-icons/file-type-python.js";
import reactIcon from "@iconify/icons-vscode-icons/file-type-reactjs.js";
import rustIcon from "@iconify/icons-vscode-icons/file-type-rust.js";
import shellIcon from "@iconify/icons-vscode-icons/file-type-shell.js";
import sqlIcon from "@iconify/icons-vscode-icons/file-type-sql.js";
import tomlIcon from "@iconify/icons-vscode-icons/file-type-toml.js";
import tsIcon from "@iconify/icons-vscode-icons/file-type-typescript.js";
import yamlIcon from "@iconify/icons-vscode-icons/file-type-yaml.js";
import excelIcon from "@iconify/icons-vscode-icons/file-type-excel.js";
import wordIcon from "@iconify/icons-vscode-icons/file-type-word.js";
import pdfIcon from "@iconify/icons-vscode-icons/file-type-pdf2.js";
import type { IconifyIcon } from "@iconify/react";

type IconModule = IconifyIcon | { default: IconifyIcon };

const EXT_ICONS: Record<string, IconModule> = {
  c: cIcon,
  cpp: cppIcon,
  css: cssIcon,
  go: goIcon,
  h: cIcon,
  html: htmlIcon,
  java: javaIcon,
  js: jsIcon,
  jsx: reactIcon,
  json: jsonIcon,
  md: markdownIcon,
  py: pythonIcon,
  rs: rustIcon,
  sh: shellIcon,
  sql: sqlIcon,
  toml: tomlIcon,
  ts: tsIcon,
  tsx: reactIcon,
  yaml: yamlIcon,
  yml: yamlIcon,
  xlsx: excelIcon,
  xls: excelIcon,
  xlsm: excelIcon,
  csv: excelIcon,
  ods: excelIcon,
  tsv: excelIcon,
  docx: wordIcon,
  pdf: pdfIcon,
};

export function FileIcon({ name, size = 18 }: { name: string; size?: number }) {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  const icon = unwrapIcon(EXT_ICONS[ext] ?? defaultFile);
  return (
    <span className="fp-icon" style={{ width: size, height: size }}>
      <Icon icon={icon} width={size} height={size} />
    </span>
  );
}

function unwrapIcon(icon: IconModule): IconifyIcon {
  return "default" in icon ? icon.default : icon;
}
