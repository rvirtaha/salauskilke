import 'htmx.org'

function Test(message: String): void {
    console.log("Hello world! ", message)
}

document.querySelectorAll('[data-action="test"]').forEach((button) => {
    button.addEventListener('click', () => Test('Click!'));
  });