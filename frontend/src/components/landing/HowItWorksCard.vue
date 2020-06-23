<template>
  <v-row class="mt-6" align="center" justify="center">
    <v-col class="text-center">
      <h2 class="display-1 font-weight-regular mb-2">How It Works</h2>
      <v-card elevation="0">
        <iframe
          :width="width"
          :height="height"
          src="https://www.youtube-nocookie.com/embed/05irUEI7yuE"
          frameborder="0"
          allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
          allowfullscreen
        ></iframe>
      </v-card>
    </v-col>
  </v-row>
</template>

<script>
export default {
  data() {
    const mobile = this.$vuetify.breakpoint.smAndDown;
    const width = mobile ? window.innerWidth - 50 : 560;
    return {
      width: width,
      height: (width / 5) * 3
    };
  },
  computed: {}
};
</script>