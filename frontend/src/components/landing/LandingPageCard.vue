<template>
  <v-row class="mt-3" align="center" justify="center">
    <v-col cols="12" md="3" :order="imgOrder">
      <v-img height="200px" contain :src="imgSrc"></v-img>
    </v-col>
    <v-col cols="12" md="6" class="text-center">
      <h2 class="headline mb-5">{{ title }}</h2>
      <p class="body-1" v-html="content"></p>
    </v-col>
  </v-row>
</template>

<script>
export default {
  props: {
    imgLeft: {
      type: Boolean,
      required: true
    },
    title: {
      type: String,
      required: true
    },
    content: {
      type: String,
      required: true
    },
    imgSrc: {
      type: String,
      required: true
    }
  },
  data() {
    return {
      mobile: this.$vuetify.breakpoint.smAndDown
    };
  },
  computed: {
    imgOrder() {
      return this.imgLeft || this.mobile ? "first" : "last";
    }
  }
};
</script>