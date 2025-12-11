import { b } from "./lit-DKg0et_P.mjs";
function escapeXml(unsafe) {
  return unsafe.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&apos;");
}
const SVG_ICON_CALENDAR = b`<!-- Adapted from https://iconoir.com/ -->
<svg width="24px" height="24px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" class="icon">
  <path d="M15 4V2M15 4V6M15 4H10.5M3 10V19C3 20.1046 3.89543 21 5 21H19C20.1046 21 21 20.1046 21 19V10H3Z" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M3 10V6C3 4.89543 3.89543 4 5 4H7" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M7 2V6" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M21 10V6C21 4.89543 20.1046 4 19 4H18.5" stroke-linecap="round" stroke-linejoin="round"></path>
</svg>
`;
const SVG_ICON_INTERNET = b`<!-- Adapted from https://iconoir.com/ -->
<svg class="icon" width="24px" height="24px" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M22 12C22 6.47715 17.5228 2 12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M13 2.04932C13 2.04932 16 5.99994 16 11.9999" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M11 21.9506C11 21.9506 8 17.9999 8 11.9999C8 5.99994 11 2.04932 11 2.04932" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M2.62964 15.5H12" stroke-linecap="round" stroke-linejoin="round"></path>
  <path d="M2.62964 8.5H21.3704" stroke-linecap="round" stroke-linejoin="round"></path>
  <path fill-rule="evenodd" clip-rule="evenodd" d="M21.8789 17.9174C22.3727 18.2211 22.3423 18.9604 21.8337 19.0181L19.2671 19.309L18.1159 21.6213C17.8878 22.0795 17.1827 21.8552 17.0661 21.2873L15.8108 15.1713C15.7123 14.6913 16.1437 14.3892 16.561 14.646L21.8789 17.9174Z"></path>
</svg>
`;
export {
  SVG_ICON_CALENDAR as S,
  SVG_ICON_INTERNET as a,
  escapeXml as e
};
