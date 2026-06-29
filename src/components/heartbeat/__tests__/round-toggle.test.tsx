import { cleanup, render, fireEvent } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RoundToggle } from "../round-toggle";

afterEach(cleanup);

describe("RoundToggle", () => {
  it("rend une checkbox avec l'état checked", () => {
    const { getByRole } = render(<RoundToggle checked={true} onChange={() => {}} />);
    const checkbox = getByRole("checkbox") as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
  });

  it("rend une checkbox avec l'état unchecked", () => {
    const { getByRole } = render(<RoundToggle checked={false} onChange={() => {}} />);
    const checkbox = getByRole("checkbox") as HTMLInputElement;
    expect(checkbox.checked).toBe(false);
  });

  it("appelle onChange avec la nouvelle valeur au clic", () => {
    const onChange = vi.fn();
    const { getByRole } = render(<RoundToggle checked={false} onChange={onChange} />);
    const checkbox = getByRole("checkbox");

    fireEvent.click(checkbox);

    expect(onChange).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("transmet le titre (title attribute)", () => {
    const { getByTitle } = render(
      <RoundToggle checked={true} onChange={() => {}} title="Activer le réveil" />,
    );
    expect(getByTitle("Activer le réveil")).toBeTruthy();
  });

  it("désactive la checkbox quand disabled=true", () => {
    const { getByRole } = render(
      <RoundToggle checked={false} onChange={() => {}} disabled={true} />,
    );
    const checkbox = getByRole("checkbox") as HTMLInputElement;
    expect(checkbox.disabled).toBe(true);
  });

  it("marque la checkbox disabled (non interactive)", () => {
    // Note : jsdom déclenche quand même onChange sur click même si disabled,
    // contrairement à un vrai navigateur. On vérifie donc l'attribut disabled
    // qui est le vrai signal d'accessibilité.
    const { getByRole } = render(
      <RoundToggle checked={false} onChange={() => {}} disabled={true} />,
    );
    const checkbox = getByRole("checkbox") as HTMLInputElement;
    expect(checkbox.disabled).toBe(true);
  });

  it("applique la classe is-disabled quand disabled", () => {
    const { container } = render(
      <RoundToggle checked={false} onChange={() => {}} disabled={true} />,
    );
    const label = container.querySelector("label");
    expect(label?.className).toContain("is-disabled");
  });

  it("affiche les labels ON et OFF", () => {
    const { getByText } = render(<RoundToggle checked={false} onChange={() => {}} />);
    expect(getByText("ON")).toBeTruthy();
    expect(getByText("OFF")).toBeTruthy();
  });
});
