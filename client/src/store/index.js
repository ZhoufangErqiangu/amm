import Vue from "vue";
import Vuex from "vuex";

Vue.use(Vuex);

export default new Vuex.Store({
  state() {
    return {
      connected: false,
      publicKey: "",
    };
  },
  mutations: {
    connect(state, wallet) {
      state.connected = wallet.connected;
      state.publicKey = wallet.publicKey.toBase58();
    },
    discontent(state, wallet) {
      state.connected = wallet.connected;
      state.publicKey = "";
    },
  },
  actions: {},
  modules: {},
});
