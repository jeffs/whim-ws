function notify(watchers, state, cache) {
  // We call all the selectors on every dispatch, but the cache saves us from
  // notifying watchers unless their selector's value has actually changed.
  const values = new Map(
    Array.from(watchers.keys(), selector => [selector, selector(state)])
  );
  for (const [selector, callbacks] of watchers.entries()) {
    const value = values.get(selector);
    if (!cache.has(selector) || cache.get(selector) !== value) {
      for (const callback of callbacks) {
        callback(value);
      }
    }
  }
  return values;
}

function reduce(reducers, state, action) {
  return Object.fromEntries(
    Object.entries(state).map(([k, v]) => [k, reducers[k](v, action)])
  );
}

function removeUnordered(haystack, needle) {
  const i = haystack.indexOf(needle);
  if (i != -1) {
    haystack[i] = haystack[haystack.length - 1];
    haystack.pop();
  }
}

export default function Store(reducers) {
  const watchers = new Map; // { [selector, [callback...]]... }

  // Cache selector results, so as not to notify watchers spuriously.
  let cache = new Map; // { [selector, value]... }

  let state = Object.fromEntries(
    Object.entries(reducers).map(([k, f]) => [k, f(undefined, Symbol())])
  );

  return {
    dispatch(action) {
      if (typeof action === 'function') {
        action(this);
      } else {
        state = reduce(reducers, state, action);
        cache = notify(watchers, state, cache);
      }
    },

    state() {
      return state;
    },

    subscribe(selector, listener) {
      if (watchers.has(selector)) {
        watchers.get(selector).push(listener);
      } else {
        watchers.set(selector, [listener]);
      }
    },

    // unsubscribe(key, listener) {
    //   if (key in listeners) {
    //     removeUnordered(listeners[key], listener);
    //   }
    // },
  };
}
