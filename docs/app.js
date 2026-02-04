const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

if (!prefersReducedMotion) {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('in-view');
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.2 }
  );

  document.querySelectorAll('[data-animate]').forEach((el) => observer.observe(el));
} else {
  document.querySelectorAll('[data-animate]').forEach((el) => el.classList.add('in-view'));
}
