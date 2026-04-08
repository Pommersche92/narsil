  /* ── Starfield & ember canvas ──────────────────────────── */
  (function () {
    const canvas = document.getElementById('cosmos');
    const ctx    = canvas.getContext('2d');
    let W, H, stars, embers, t = 0;

    function resize() {
      W = canvas.width  = window.innerWidth;
      H = canvas.height = window.innerHeight;
    }

    function rnd(a, b) { return a + Math.random() * (b - a); }

    function mkStar() {
      return { x: Math.random()*W, y: Math.random()*H,
               r: rnd(0.3,1.3), a: rnd(0.2,0.85),
               phase: Math.random()*Math.PI*2, speed: rnd(200,800) };
    }

    function mkEmber(fromBottom) {
      return { x: rnd(W*0.1, W*0.9),
               y: fromBottom ? H + rnd(0,20) : rnd(0, H),
               r: rnd(0.7, 2.2), a: rnd(0.35,0.8),
               vx: rnd(-0.25, 0.25), vy: rnd(-0.7,-0.2),
               life: 1, decay: rnd(0.0018,0.0045),
               hue: rnd(18, 46) };
    }

    function init() {
      stars  = Array.from({length: 180}, mkStar);
      embers = Array.from({length: 30},  () => mkEmber(false));
    }

    function frame() {
      ctx.clearRect(0, 0, W, H);
      t += 0.016;

      for (const s of stars) {
        const a = s.a * (0.65 + 0.35 * Math.sin(t * 1000 / s.speed + s.phase));
        ctx.beginPath();
        ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(210,195,155,${a})`;
        ctx.fill();
      }

      for (let i = embers.length - 1; i >= 0; i--) {
        const e = embers[i];
        e.x    += e.vx + rnd(-0.015, 0.015);
        e.y    += e.vy;
        e.life -= e.decay;
        if (e.life <= 0 || e.y < -12) { embers[i] = mkEmber(true); continue; }
        const a = e.a * e.life;
        ctx.beginPath();
        ctx.arc(e.x, e.y, e.r, 0, Math.PI*2);
        ctx.fillStyle = `hsla(${e.hue},90%,65%,${a})`;
        ctx.fill();
        ctx.beginPath();
        ctx.arc(e.x, e.y, e.r*3.5, 0, Math.PI*2);
        ctx.fillStyle = `hsla(${e.hue},90%,65%,${a*0.12})`;
        ctx.fill();
      }

      requestAnimationFrame(frame);
    }

    resize();
    init();
    frame();
    window.addEventListener('resize', () => { resize(); init(); });
  })();

  /* ── Scroll reveal ─────────────────────────────────────── */
  const io = new IntersectionObserver(entries => {
    entries.forEach(e => {
      if (e.isIntersecting) { e.target.classList.add('visible'); io.unobserve(e.target); }
    });
  }, { threshold: 0.1 });
  document.querySelectorAll('.fade-in-up').forEach(el => io.observe(el));

  /* ── Tab switcher ──────────────────────────────────────── */
  function switchTab(id, btn) {
    document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    document.getElementById('tab-' + id).classList.add('active');
    btn.classList.add('active');
  }

  /* ── DSGVO banner ────────────────────────────────────────── */
  (function () {
    const banner = document.getElementById('dsgvo-banner');
    const btn    = document.getElementById('dsgvo-accept');
    if (!banner) return;
    if (!localStorage.getItem('dsgvo-accepted')) {
      banner.removeAttribute('hidden');
    }
    btn.addEventListener('click', function () {
      localStorage.setItem('dsgvo-accepted', '1');
      banner.setAttribute('hidden', '');
    });
  })();

  /* ── Copy code ─────────────────────────────────────────── */
  function copyCode(btn, text) {
    if (!navigator.clipboard) return;
    navigator.clipboard.writeText(text).then(() => {
      const prev = btn.textContent;
      btn.textContent = 'copied!';
      btn.style.color = '#5abf72';
      setTimeout(() => { btn.textContent = prev; btn.style.color = ''; }, 1600);
    });
  }
