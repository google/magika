import vuetify from "./vuetify.js";

// Note: no need to do createApp; this integration already gives you an
// instantiated app.
export default (app) => {
  app.use(vuetify);
};
