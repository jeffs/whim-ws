const ROUTE_PREFIX = "/api/v0/";

export default {
  postName(name) {
    return fetch(`${ROUTE_PREFIX}/name`, { method: 'POST' });
  },
};
