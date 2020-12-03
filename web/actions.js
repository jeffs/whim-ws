import host from './host.js';

export const POST_NAME = Symbol('POST_NAME');
export const SET_NAME = Symbol('SET_NAME');

export function PostName(name) {
  return store => {
    host.postName(name)
      .then(() => console.log('success'))
      .catch(err => console.log(err));
  };
}

export function SetName(name) {
  return {
    type: SET_NAME,
    name,
  };
}
