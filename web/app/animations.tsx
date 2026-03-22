"use client";

import { useEffect } from "react";

export function Animations() {
  useEffect(() => {
    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (prefersReducedMotion) return;

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

    const sections = document.querySelectorAll(
      "#features, #stealth-layers, #how-it-works, #shortcuts, #download"
    );
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

    requestAnimationFrame(() => {
      document.querySelectorAll("#hero [data-animate]").forEach((el) => {
        el.classList.add("is-visible");
      });
    });

    return () => {
      elementObserver.disconnect();
      sectionObserver.disconnect();
    };
  }, []);

  return null;
}

export function HeroGlow() {
  useEffect(() => {
    const hero = document.getElementById("hero");
    const glow = document.querySelector(".hero-glow");

    if (!hero || !glow) return;

    const glowEl = glow instanceof HTMLElement ? glow : null;
    if (!glowEl) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = hero.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      glowEl.style.left = x + "px";
      glowEl.style.top = y + "px";
    };

    const handleMouseLeave = () => {
      glowEl.style.left = "50%";
      glowEl.style.top = "50%";
    };

    hero.addEventListener("mousemove", handleMouseMove);
    hero.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      hero.removeEventListener("mousemove", handleMouseMove);
      hero.removeEventListener("mouseleave", handleMouseLeave);
    };
  }, []);

  return null;
}
