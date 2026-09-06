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