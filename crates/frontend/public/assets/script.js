// Theme switching
const themeSelect = document.getElementById("theme");
themeSelect.addEventListener("change", (e) => {
  localStorage.setItem("theme", e.target.value);
  document.body.className = e.target.value;
  
  // set prefers color scheme to match the theme
  console.log("prefers-color-scheme", window.mediaMatches);
  document.documentElement.setAttribute("data-theme", e.target.value);
});

const savedTheme = localStorage.getItem("theme");
if (savedTheme) {
  themeSelect.value = savedTheme;
  document.body.className = savedTheme;
  document.documentElement.setAttribute("data-theme", savedTheme);
} else {
  let isDarkMode = window.matchMedia("(prefers-color-scheme: dark)").matches;
  if (isDarkMode) {
    document.documentElement.setAttribute("data-theme", "modern-dark");
  } else {
    document.documentElement.setAttribute("data-theme", "modern-light");
  }
  localStorage.setItem("theme", document.body.className);
}
