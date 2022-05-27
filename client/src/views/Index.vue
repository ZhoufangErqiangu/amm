<template>
  <div class="index-box">
    <find />
    <create />
  </div>
</template>

<script>
import Find from "./components/Find.vue";
import Create from "./components/Create.vue";

export default {
  name: "Index",
  components: {
    Find,
    Create,
  },
  data() {
    return {};
  },
  methods: {},
  mounted() {},
  computed: {},
};
</script>

<style lang="less" scoped></style>
