<template>
  <div class="index-box">
    <el-card>
      <div slot="header">
        <span>Post List</span>
        <el-button
          class="ml15"
          size="mini"
          type="primary"
          :loading="loading"
          @click="onFetch"
        >
          Fetch
        </el-button>
      </div>
      <div class="none" v-show="list.length === 0">No Data</div>
      <div class="list" v-show="list.length > 0">
        <post-unit v-for="(item, index) in list" :key="index" :data="item" />
      </div>
    </el-card>
    <el-card class="mt25">
      <div slot="header">
        <span>Send Post</span>
        <el-button
          class="ml15"
          size="mini"
          type="primary"
          :loading="loadingS"
          @click="onSend"
          :disabled="!isConnected"
        >
          Send
        </el-button>
      </div>
      <div class="input">
        <el-input
          placeholder="Input message"
          v-model="content"
          type="textarea"
        />
        <div class="count">Input count: {{ textCount }}</div>
      </div>
    </el-card>
  </div>
</template>

<script>
import PostUnit from "./components/PostUnit.vue";
import { Connection } from "@solana/web3.js";
import { wallet } from "../plugin/wallet";
import { findAllMasterPost, findShardingPost, post } from "../assets/js";
import store from "@/store";
var utils;
import("../assets/js/pkg").then((module) => {
  utils = module;
  console.log("wasm load ok", module);
});

export default {
  name: "Index",
  components: {
    PostUnit,
  },
  data() {
    return {
      connection: new Connection("http://localhost:8899"),
      loading: false,
      list: [],
      loadingS: false,
      content: "",
    };
  },
  methods: {
    async onFetch() {
      utils.greet();
      this.loading = true;
      this.list.length = 0;
      try {
        let list = await findAllMasterPost(this.connection);
        if (list.length) {
          list.sort((a, b) => {
            return b.time - a.time;
          });
          this.list = list;
        }
        for (let i = 0; i < list.length; i++) {
          let shardingList = await findShardingPost(
            this.connection,
            list[i].key
          );
          if (shardingList.length != 0) {
            shardingList.sort((a, b) => {
              return a.sharding - b.sharding;
            });
            shardingList.forEach((e) => {
              list[i].content += e.content;
            });
          }
        }
      } catch (err) {
        this.$message({ message: "fetch error", type: "error" });
        console.error("get post data error", err);
      }
      this.loading = false;
    },
    async onSend() {
      this.loadingS = true;
      try {
        if (wallet.connected) {
          let res = await post(this.connection, wallet, this.content);
          if (res.code == 1) {
            this.content = "";
            this.onFetch();
            this.$message({ message: "send success", type: "success" });
          } else {
            this.$message({ message: "send fail", type: "warning" });
            console.warn("send fail", res);
          }
        }
      } catch (err) {
        this.$message({ message: "send error", type: "error" });
        console.error("send error", err);
      }
      this.loadingS = false;
    },
  },
  mounted() {
    this.onFetch();
  },
  computed: {
    textCount() {
      return this.content.length;
    },
    isConnected() {
      return store.state.connected;
    },
  },
};
</script>

<style lang="less" scoped>
.index-box {
  .none {
    font-size: 24px;
    font-weight: 500;
    text-align: center;
    line-height: 60px;
  }
  .input {
    .count {
      color: #777;
      text-align: end;
    }
  }
}
</style>
