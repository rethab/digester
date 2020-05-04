<template>
  <v-row class="mt-6" align="center" justify="center">
    <v-col class="text-center">
      <h1 class="display-1 font-weight-regular">How It Works</h1>
      <p
        class="mt-3 title font-weight-regular font-italic"
      >Create Subscription. Receive Digests. Relax.</p>
      <v-card elevation="12">
        <v-carousel light show-arrows-on-hover hide-delimiter-background cycle :height="height">
          <v-carousel-item>
            <v-img contain :src="searchImgSrc"></v-img>
          </v-carousel-item>
          <v-carousel-item>
            <v-img contain :src="subImgSrc"></v-img>
          </v-carousel-item>
          <v-carousel-item>
            <v-img contain :src="digestImgSrc"></v-img>
          </v-carousel-item>
        </v-carousel>
      </v-card>
    </v-col>
  </v-row>
</template>

<script>
export default {
  data() {
    return {
      mobile: this.$vuetify.breakpoint.smAndDown
    };
  },
  computed: {
    suffix() {
      return this.mobile ? "mobile" : "desktop";
    },
    searchImgSrc() {
      return require("@/assets/landing-search-" + this.suffix + ".png");
    },
    subImgSrc() {
      return require("@/assets/landing-sub-" + this.suffix + ".png");
    },
    digestImgSrc() {
      return require("@/assets/landing-digest-" + this.suffix + ".png");
    },
    height() {
      const small = this.$vuetify.breakpoint.mdAndDown;
      return small ? "400" : "700";
    }
  }
};
</script>