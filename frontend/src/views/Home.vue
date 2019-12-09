<template>
  <div class="home">
    <h1 class="display-3 d-flex justify-center">Blogs</h1>
    <div class="blogs-container">
      <div v-for="blog in blogs" :key="blog.url">
        <h3>{{blog.url}}</h3>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "home",
  components: {},
  computed: {
    blogs() {
      return this.$store.state.blogs;
    }
  },
  mounted() {
    this.$store.dispatch("loadBlogs");
  }
};
</script>
