import { describe, expect, it } from "vitest";
import { countLines, fileNameFromPath, shortPath } from "./file-preview-utils";

describe("fileNameFromPath", () => {
  it("extrait le nom de fichier depuis un chemin Unix", () => {
    expect(fileNameFromPath("/home/user/project/main.ts")).toBe("main.ts");
    expect(fileNameFromPath("/src/lib/utils.ts")).toBe("utils.ts");
  });

  it("extrait le nom de fichier depuis un chemin Windows (backslash)", () => {
    expect(fileNameFromPath("C:\\Users\\Kevin\\project\\index.ts")).toBe("index.ts");
    expect(fileNameFromPath("C:\\src\\main.rs")).toBe("main.rs");
  });

  it("retourne le chemin tel quel si pas de séparateur", () => {
    expect(fileNameFromPath("fichier.ts")).toBe("fichier.ts");
    expect(fileNameFromPath("")).toBe("");
  });

  it("gère les chemins avec séparateurs mixtes", () => {
    expect(fileNameFromPath("/home/user\\project/file.ts")).toBe("file.ts");
  });
});

describe("shortPath", () => {
  it("strip le baseDir du début du chemin", () => {
    expect(shortPath("/home/user/project/src/main.ts", "/home/user/project")).toBe("src/main.ts");
  });

  it("normalise les backslashes Windows dans le chemin", () => {
    expect(shortPath("C:\\Users\\Kevin\\project\\src\\main.ts", "C:\\Users\\Kevin\\project")).toBe("src/main.ts");
  });

  it("retourne le chemin original si baseDir est absent", () => {
    expect(shortPath("/home/user/file.ts")).toBe("/home/user/file.ts");
    expect(shortPath("/home/user/file.ts", "")).toBe("/home/user/file.ts");
  });

  it("retourne le chemin original si le baseDir ne correspond pas", () => {
    expect(shortPath("/other/path/file.ts", "/home/user/project")).toBe("/other/path/file.ts");
  });

  it("tolère un baseDir avec slash final", () => {
    expect(shortPath("/home/user/project/file.ts", "/home/user/project/")).toBe("file.ts");
  });
});

describe("countLines", () => {
  it("compte les lignes séparées par \\n", () => {
    expect(countLines("ligne1\nligne2\nligne3")).toBe(3);
  });

  it("compte les lignes séparées par \\r\\n (Windows)", () => {
    expect(countLines("ligne1\r\nligne2\r\nligne3")).toBe(3);
  });

  it("retourne 1 pour une chaîne sans saut de ligne", () => {
    expect(countLines("une seule ligne")).toBe(1);
  });

  it("retourne 0 pour une valeur falsy", () => {
    expect(countLines(undefined)).toBe(0);
    expect(countLines("")).toBe(0);
  });
});

describe("fileNameFromPath — cas limites supplémentaires", () => {
  it("chemin vide retourne une chaîne vide", () => {
    expect(fileNameFromPath("")).toBe("");
  });
});

describe("shortPath — cas limites supplémentaires", () => {
  it("baseDir qui ne matche pas du tout retourne le chemin original", () => {
    expect(shortPath("/home/user/file.ts", "/srv/data")).toBe("/home/user/file.ts");
  });

  it("baseDir avec trailing slash est correctement géré", () => {
    expect(shortPath("/home/user/project/file.ts", "/home/user/project/")).toBe("file.ts");
  });
});

describe("countLines — cas limites supplémentaires", () => {
  it("retourne 0 pour undefined", () => {
    expect(countLines(undefined)).toBe(0);
  });

  it("retourne 0 pour chaîne vide (falsy)", () => {
    expect(countLines("")).toBe(0);
  });

  it("compte correctement les lignes avec retours chariot Windows (\\r\\n)", () => {
    expect(countLines("ligne1\r\nligne2\r\nligne3")).toBe(3);
  });
});
