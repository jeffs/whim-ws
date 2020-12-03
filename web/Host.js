// Represents a Whim API server.

export default function Host(url = "", prefix = "/v0") {
  return {
    getDefaultShell() {
      console.log('getDefaultShell'); // TODO
      return fetch(`${prefix}/shell/0`);
    },

    postName(name) {
      return fetch(`${prefix}/name`, { method: 'POST' });
    },
  };
}
