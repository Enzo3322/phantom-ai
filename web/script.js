// Smooth scroll
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
  anchor.addEventListener("click", function (e) {
    e.preventDefault();
    const target = document.querySelector(this.getAttribute("href"));
    if (target) {
      target.scrollIntoView({ behavior: "smooth" });
    }
  });
});

// Hero glow follows cursor
const hero = document.getElementById("hero");
const glow = document.querySelector(".hero-glow");

if (hero && glow) {
  hero.addEventListener("mousemove", (e) => {
    const rect = hero.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    glow.style.left = x + "px";
    glow.style.top = y + "px";
  });

  hero.addEventListener("mouseleave", () => {
    glow.style.left = "50%";
    glow.style.top = "50%";
  });
}

// Scroll-triggered animations
const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

if (!prefersReducedMotion) {
  // Animate individual elements
  const animatedElements = document.querySelectorAll("[data-animate]");
  const elementObserver = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("is-visible");
          elementObserver.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.15, rootMargin: "0px 0px -40px 0px" }
  );

  animatedElements.forEach((el) => elementObserver.observe(el));

  // Animate section divider lines
  const sections = document.querySelectorAll("#features, #how-it-works, #shortcuts, #download");
  const sectionObserver = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("section-visible");
          sectionObserver.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.05 }
  );

  sections.forEach((s) => sectionObserver.observe(s));

  // Hero elements animate on load (already in viewport)
  requestAnimationFrame(() => {
    document.querySelectorAll("#hero [data-animate]").forEach((el) => {
      el.classList.add("is-visible");
    });
  });
}
