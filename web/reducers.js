import * as actions from './actions.js';

export function name(state = '', action) {
  return action.type === actions.SET_NAME
    ? action.name
    : state;
}
