/* CL-GO DASH — Mockup Interactions */

const lp = document.getElementById('lp');
const dp = document.getElementById('dp');

function sw(tab) {
  document.querySelectorAll('.nav-item').forEach(e => e.classList.remove('active'));
  document.querySelector(`[data-tab="${tab}"]`).classList.add('active');
  lp.innerHTML = LISTS[tab];
  dp.innerHTML = DETAILS[tab];
  bindModes();
}

function tt() {
  const h = document.documentElement;
  const i = document.getElementById('ti');
  const l = document.getElementById('tl');
  if (h.dataset.theme === 'dark') {
    h.dataset.theme = 'light';
    i.textContent = '☀️';
    l.textContent = 'Light mode';
  } else {
    h.dataset.theme = 'dark';
    i.textContent = '🌙';
    l.textContent = 'Dark mode';
  }
}

function ctx(e) {
  e.preventDefault();
  const m = document.getElementById('cm');
  m.style.left = e.clientX + 'px';
  m.style.top = e.clientY + 'px';
  m.classList.add('visible');
}

function showW() {
  dp.innerHTML = DETAILS.warnings;
}

function bindModes() {
  document.querySelectorAll('.mode-option').forEach(o => {
    o.addEventListener('click', () => {
      document.querySelectorAll('.mode-option').forEach(x => x.classList.remove('active'));
      o.classList.add('active');
    });
  });
  document.querySelectorAll('.toggle').forEach(t => {
    t.addEventListener('click', () => t.classList.toggle('on'));
  });
}

document.addEventListener('click', () => {
  document.getElementById('cm').classList.remove('visible');
});

/* Init — load heartbeat view */
sw('heartbeat');
