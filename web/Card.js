function headerText(value) {
  return value.trim() || 'card';
}

// Note: The name could just as well be local state.
export default function Card(
  store,
  selector,
  inputActionCreator,
  submitActionCreator,
) {
  const header = document.createElement('h1');
  header.classList.add('card__header');
  header.innerText = `Hello, ${headerText(selector(store.state()))}.`;

  const input = document.createElement('input');
  input.type = 'text';
  input.addEventListener('input', event => {
    store.dispatch(inputActionCreator(event.target.value));
  });

  const form = document.createElement('form');
  form.classList.add('card');
  form.append(header, input);
  form.addEventListener('submit', async event => {
    event.preventDefault();
    store.dispatch(submitActionCreator(event.target.value));
  });

  store.subscribe(selector, value => {
    input.value = value;
    header.innerText = `Hello, ${headerText(value)}.`;
  });

  return {
    focus() {
      input.focus();
    },

    render() {
      return form;
    },
  };
}
