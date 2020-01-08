<template>
  <v-container>
    <v-row>
      <v-col cols="12" md="6">
        <p>Please login using one of the social providers.</p>
        <p class="overline grey--text">
          <span class="font-weight-bold">Note on Privacy:</span>&nbsp;
          <span>From the login provider, we are going to retrieve your e-mail address and store it in our database. We use that e-mail address to send digests to. This information is not shared with any third party whatsoever. Period.</span>
        </p>
      </v-col>
      <v-col cols="12" md="6" :order="this.mobile ? 'first' : 'last'">
        <h2 class="title">Available methods*</h2>
        <p class="font-italic caption">* More coming soon</p>
        <GithubLoginBtn />
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
import GithubLoginBtn from "@/components/auth/GithubLoginBtn.vue";

export default {
  name: "auth-login",
  components: {
    GithubLoginBtn
  },
  data() {
    return {
      mobile: this.$vuetify.breakpoint.smAndDown,

      message: this.$route.query.requireAuth
        ? "This page requires authentication. Please log in."
        : this.$route.query.sessionExpired
        ? "Your session has expired. Please login again."
        : null
    };
  }
};
</script>