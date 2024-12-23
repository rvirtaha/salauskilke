// These are imported to include them in the vite build
import 'htmx.org'
import './tailwind.css'

function Test(message: String): void {
    console.log("Hello world! ", message)
}

document.querySelectorAll('[data-action="test"]').forEach((button) => {
    button.addEventListener('click', () => Test('Click!'));
  });