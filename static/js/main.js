document.addEventListener('DOMContentLoaded', () => {
  const dismissFlash = (flash) => {
    flash.classList.remove('show');
    setTimeout(() => flash.remove(), 300);
  };

  document.querySelectorAll('.flash').forEach((flash) => {
    // إغلاق تلقائي بعد 5 ثوانٍ
    const timer = setTimeout(() => dismissFlash(flash), 5000);

    // إغلاق يدوي عند الضغط على الزر
    const closeBtn = flash.querySelector('.flash__close');
    if (closeBtn) {
      closeBtn.addEventListener('click', () => {
        clearTimeout(timer); // إيقاف العداد التلقائي
        dismissFlash(flash);
      });
    }
  });
});

/* ==========================================================================
   3. Global Outside Click Handler (Dropdowns & Active Overlays)
   ========================================================================== */

window.addEventListener('click', (e) => {
  // Close any search results dropdown when clicking outside the container
  const searchContainer = document.querySelector('.search-container');
  const searchResults = document.getElementById('search-results');
  
  if (searchContainer && searchResults && !searchContainer.contains(e.target)) {
    searchResults.innerHTML = '';
  }
});





document.addEventListener('click', function (event) {
    const searchContainer = document.querySelector('.search-container');
    const dropdown = document.getElementById('search-results-dropdown');

    // إذا كانت القائمة موجودة والنقر تم خارج .search-container
    if (dropdown && searchContainer && !searchContainer.contains(event.target)) {
        dropdown.innerHTML = '';
    }
});

// إغلاق القائمة عند الضغط على زر Escape
document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') {
        const dropdown = document.getElementById('search-results-dropdown');
        if (dropdown) dropdown.innerHTML = '';
    }
});
