// TODO: Dispatch actions and thunks, not key/value pairs.
// TODO: Pass actions to reducers instead of updating state directly.

function Store() {
  let state = { };
  const listeners = { };  // key => [listeners...]

  return {
    dispatch(key, value) {
      state = { ...state, [key]: value };
      if (key in listeners) {
        for (const listener of listeners[key]) {
          listener(value);
        }
      }
    },

    subscribe(key, listener) {
      if (key in listeners) {
        listeners[key].push(listener);
      } else {
        listeners[key] = [listener];
      }
    },

    unsubscribe(key, listener) {
      if (key in listeners) {
        let i = listeners[key].indexOf(listener);
        if (i != -1) {
          listeners[key][i] = listeners[key][listeners.length - 1];
          listeners[key].pop();
        }
      }
    },
  };
}

function Greeter(store, nameKey) {
  const header = document.createElement('h1');
  header.innerText = 'Hello, world.';

  const input = document.createElement('input');
  input.type = 'text';
  input.addEventListener('input', event => {
    store.dispatch(nameKey, event.target.value);
  });

  const form = document.createElement('form');
  form.append(header, input);

  store.subscribe(nameKey, value => {
    input.value = value;
    header.innerText = `Hello, ${input.value.trim() || 'world'}.`;
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

async function main() {
  const store = Store();
  const greeter = Greeter(store, 'whom');
  document.body.append(greeter.render());
  greeter.focus();
}

main();
