import Vue from "vue";
import VueRouter from "vue-router";
import AppIndex from "../views/Index.vue";

const defaultTitle = "AMM";
const vueConfig = require("../../vue.config");

Vue.use(VueRouter);

const routes = [
  {
    path: "/",
    name: "AppIndex",
    component: AppIndex,
    meta: {
      title: "Index",
    },
  },
];

const router = new VueRouter({
  mode: "history",
  base: vueConfig.publicPath,
  routes,
});

console.log("router", router);

router.beforeEach((to, _from, next) => {
  console.log("route to", to);
  let title;
  if (to.meta && to.meta.title) {
    title = `${defaultTitle} | ${to.meta.title}`;
  } else {
    title = defaultTitle;
  }
  window.document.title = title;
  next();
});

export default router;
