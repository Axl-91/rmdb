document.addEventListener('DOMContentLoaded', function() {
  const rangeInput = document.getElementById('scoreRange');
  const scoreValue = document.getElementById('scoreValue');

  if (rangeInput){
    scoreValue.textContent = rangeInput.value;

    rangeInput.addEventListener('input', function() {
      scoreValue.textContent = rangeInput.value;
    });
  }
});
