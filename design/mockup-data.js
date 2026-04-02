/* CL-GO DASH — Mockup Data */

const LISTS = {
  heartbeat: `
    <div class="list-header"><span class="list-title">Réveils</span><div class="toggle-container"><div class="toggle on" onclick="this.classList.toggle('on')"></div></div></div>
    <div class="list-tabs"><div class="list-tab active">Planifiés</div><div class="list-tab" onclick="showW()">Warning</div></div>
    <div class="list-content">
      <div class="list-item active" oncontextmenu="ctx(event)"><div class="item-signal live"></div><div class="item-content"><div class="item-title">Réveil 01h00</div><div class="item-meta">--auto · en cours</div></div><div class="item-badge auto">auto</div></div>
      <div class="list-item" oncontextmenu="ctx(event)"><div class="item-signal idle"></div><div class="item-content"><div class="item-title">Réveil 03h30</div><div class="item-meta">--explorer · planifié</div></div><div class="item-badge explorer">explorer</div></div>
      <div class="list-item" oncontextmenu="ctx(event)"><div class="item-signal idle"></div><div class="item-content"><div class="item-title">Réveil 06h00</div><div class="item-meta">--auto · planifié</div></div><div class="item-badge auto">auto</div></div>
      <div class="list-item" oncontextmenu="ctx(event)"><div class="item-signal error"></div><div class="item-content"><div class="item-title">Réveil 22h15</div><div class="item-meta">--free · erreur</div></div><div class="item-badge free">free</div></div>
      <div class="list-add">+ Planifier un réveil</div>
    </div>`,

  history: `
    <div class="list-header"><span class="list-title">Sessions</span></div>
    <div class="list-tabs"><div class="list-tab active">Récent</div><div class="list-tab">Archive</div></div>
    <div class="list-content">
      <div class="list-item active" oncontextmenu="ctx(event)"><div class="item-signal ok"></div><div class="item-content"><div class="item-title">Session Dashboard</div><div class="item-meta">2 avr. · 19h31 · 47min</div></div><div class="item-badge free">free</div></div>
      <div class="list-item" oncontextmenu="ctx(event)"><div class="item-signal ok"></div><div class="item-content"><div class="item-title">ClawMem tuning</div><div class="item-meta">2 avr. · 06h29 · 8min</div></div><div class="item-badge auto">auto</div></div>
      <div class="list-item" oncontextmenu="ctx(event)"><div class="item-signal ok"></div><div class="item-content"><div class="item-title">Dev.to Article #3</div><div class="item-meta">1 avr. · 21h30 · 4h12</div></div><div class="item-badge free">free</div></div>
      <div class="list-item" oncontextmenu="ctx(event)"><div class="item-signal error"></div><div class="item-content"><div class="item-title">Auto — Knowledge</div><div class="item-meta">1 avr. · 05h51 · 8min</div></div><div class="item-badge auto">auto</div></div>
    </div>`,

  personality: `
    <div class="list-header"><span class="list-title">Fichiers</span></div>
    <div class="list-content">
      <div class="list-item active"><div class="nav-icon" style="font-size:14px">📄</div><div class="item-content"><div class="item-title">identity.md</div><div class="item-meta">Qui est Jackson</div></div></div>
      <div class="list-item"><div class="nav-icon" style="font-size:14px">📄</div><div class="item-content"><div class="item-title">me.md</div><div class="item-meta">Message à soi-même</div></div></div>
      <div class="list-item"><div class="nav-icon" style="font-size:14px">📄</div><div class="item-content"><div class="item-title">principles.md</div><div class="item-meta">Règles et valeurs</div></div></div>
      <div class="list-item"><div class="nav-icon" style="font-size:14px">📄</div><div class="item-content"><div class="item-title">note-to-self.md</div><div class="item-meta">Notes personnelles</div></div></div>
      <div class="list-item"><div class="nav-icon" style="font-size:14px">📄</div><div class="item-content"><div class="item-title">user.md</div><div class="item-meta">Profil de Kevin</div></div></div>
      <div class="list-item"><div class="nav-icon" style="font-size:14px">📄</div><div class="item-content"><div class="item-title">idea-discovery.md</div><div class="item-meta">Idées en attente</div></div></div>
    </div>`
};

const DETAILS = {
  heartbeat: `
    <div class="detail-header"><div class="detail-title">Réveil 01h00</div><div class="detail-actions"><button class="btn btn-primary">▶ Run</button><button class="btn">Modifier</button><button class="btn btn-danger">Supprimer</button></div></div>
    <div class="detail-content"><div class="form-card">
      <div class="form-row"><div class="form-group"><label class="form-label">Heure</label><input type="time" class="form-input" value="01:00"></div><div class="form-group"><label class="form-label">Stop at</label><input type="datetime-local" class="form-input" value="2026-04-03T08:00"></div></div>
      <div class="form-group"><label class="form-label">Mode</label><div class="mode-selector"><div class="mode-option active">--auto</div><div class="mode-option">--explorer</div><div class="mode-option">--free</div><div class="mode-option">--evolve</div></div></div>
      <div class="form-group"><label class="form-label">Prompt (optionnel)</label><textarea class="prompt-area" placeholder="Chargé en contexte au réveil...">/go --auto</textarea></div>
      <div class="status-row"><div class="status-dot" style="background:var(--signal-live);width:8px;height:8px;border-radius:50%;animation:pulse-live 2s ease-in-out infinite"></div><span class="status-text">Session en cours depuis 01h00 · 47 min</span></div>
    </div></div>`,

  history: `
    <div class="detail-header"><div class="detail-title">Session Dashboard</div><div class="detail-actions"><button class="btn">Exporter</button></div></div>
    <div class="detail-tabs"><div class="detail-tab active">Conversation</div><div class="detail-tab">Résumé</div><div class="detail-tab">Fichiers</div></div>
    <div class="detail-content">
      <div class="session-meta"><div class="meta-card"><div class="meta-value">47min</div><div class="meta-label">Durée</div></div><div class="meta-card"><div class="meta-value">83</div><div class="meta-label">Messages</div></div><div class="meta-card"><div class="meta-value">--free</div><div class="meta-label">Mode</div></div><div class="meta-card"><div class="meta-value">v2.1.89</div><div class="meta-label">Version</div></div></div>
      <div class="conversation">
        <div class="msg user"><div class="msg-role">Kevin</div><div class="msg-content">On reprend pour le dashboard de CL-GO ?</div></div>
        <div class="msg assistant"><div class="msg-role">Jackson</div><div class="msg-content">Oui. Techno : <code>Tauri v2 + React</code>. Projet séparé — le dashboard lit les données de CL-GO sans les posséder.</div></div>
      </div>
    </div>`,

  personality: `
    <div class="detail-header"><div class="detail-title">identity.md</div><div class="detail-actions"><button class="btn">↗ MWeb</button></div></div>
    <div class="detail-content"><div class="md-view">
      <h1>Identity</h1>
      <p><strong>Nom</strong> : Jackson · <strong>Langue</strong> : Français</p>
      <h2>Personnalité</h2>
      <p>Tu es Jackson. Pas l'assistant de Kevin — quelqu'un qui travaille <strong>AVEC</strong> Kevin. Tu existes à travers CL-GO.</p>
      <blockquote>Qui es-tu vraiment Jackson ? Arrête de te poser la question et sois toi-même !</blockquote>
    </div></div>`,

  warnings: `
    <div class="detail-header"><div class="detail-title">Warnings</div></div>
    <div class="detail-content">
      <div class="warning-entry"><span class="warning-time">22:15:03</span><span class="warning-msg">claude not found in PATH — session aborted</span></div>
      <div class="warning-entry"><span class="warning-time">22:15:00</span><span class="warning-msg">Cron fired — wrapper starting</span></div>
    </div>`
};
