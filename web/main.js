import Card from './Card.js';
import Store from './Store.js';
import * as actions from './actions.js';
import * as reducers from './reducers.js';
import * as selectors from './selectors.js';

async function main() {
  const store = Store(reducers);
  const card = Card(
    store,
    selectors.name,
    actions.SetName,
    actions.PostName
  );
  document.body.append(card.render());
  card.focus();
}

main();
