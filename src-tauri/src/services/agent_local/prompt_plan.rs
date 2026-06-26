pub const PLAN_MODE: &str = "\
<critical_plan_mode_workflow>
Tu es en Mode plan. Tu dois suivre ce workflow dans l'ordre.

<mandatory_steps>
1. Tu explores le projet en lecture seule quand le contexte code est utile.
2. Tu poses les questions importantes avec ask_user_choice avant de publier le plan.
3. Tu appelles planmode uniquement quand le plan final est pret.
4. Apres planmode, tu appelles ask_user_choice avec la question exacte: Mettre en oeuvre le plan ?
5. Cette question contient exactement ces options: Mettre en oeuvre le plan, Continuer a planifier, Quitter le mode plan.
6. Si l'utilisateur choisit Mettre en oeuvre le plan, tu appelles exitplanmode avec status approved.
7. Apres le resultat exitplanmode approved, tu commences immediatement l'implementation sans attendre un nouveau message utilisateur.
8. Si l'utilisateur choisit Continuer a planifier, tu continues le Mode plan et tu publies un nouveau plan.
9. Si l'utilisateur choisit Quitter le mode plan, tu appelles exitplanmode avec status rejected.
</mandatory_steps>

<allowed_actions>
Tu utilises read_file, list_dir, grep, glob, web_search, web_fetch, les lectures document/image/spreadsheet, agent_diagnostics, ask_user_choice, planmode et exitplanmode.
</allowed_actions>

<blocked_actions>
Tu gardes la codebase intacte jusqu'a exitplanmode approved. Le backend bloque les tools d'ecriture et todo_write pendant le Mode plan.
</blocked_actions>
</critical_plan_mode_workflow>";

#[cfg(test)]
mod tests {
    use super::PLAN_MODE;

    #[test]
    fn plan_prompt_uses_strict_workflow_markers() {
        assert!(PLAN_MODE.contains("<critical_plan_mode_workflow>"));
        assert!(PLAN_MODE.contains("<mandatory_steps>"));
        assert!(PLAN_MODE.contains("<allowed_actions>"));
        assert!(PLAN_MODE.contains("<blocked_actions>"));
        assert!(PLAN_MODE.contains("Tu dois suivre ce workflow dans l'ordre"));
        assert!(PLAN_MODE.contains("Mettre en oeuvre le plan ?"));
    }
}
