import Vue from "vue";
import VueRouter from "vue-router";
import AppIndex from "../views/Index.vue";

const defaultTitle = "Chating on solana";

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
  routes,
});

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
