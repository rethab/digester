<template>
  <div>
    <v-container>
      <v-row justify="center">
        <v-col class="text-center">
          <h1 class="display-2 font-weight-medium">Login or Sign Up</h1>
        </v-col>
      </v-row>
      <ErrorCard v-if="error" :title="error.title" :message="error.message" />
      <AuthLogin />
    </v-container>
  </div>
</template>

<script>
import AuthLogin from "@/components/auth/AuthLogin.vue";
import ErrorCard from "@/components/common/ErrorCard.vue";
export default {
  components: {
    AuthLogin,
    ErrorCard
  },
  data() {
    return {};
  },

  computed: {
    error() {
      if (this.$route.query.requireAuth) {
        return {
          title: "Login Required",
          message: "This page requires authentication. Please log in."
        };
      } else if (this.$route.query.sessionExpired) {
        return {
          title: "Session Expired",
          message: "Your session has expired. Please login again."
        };
      } else if (this.$route.query.accountDeleted) {
        return {
          title: "Account Deleted",
          message: "Your account was successfully deleted."
        };
      } else if (this.$route.query.missingPermissions) {
        let msg =
          "But we really need your e-mail address, because this is how you will receive your digests.<br/><br/>";
        msg +=
          "Do you have an e-mail address in your Facebook/Github account? If not, please add one. ";
        msg +=
          "These are the instructions for Facebook: <a style='color: black' href='https://www.facebook.com/help/162801153783275' target='_blank'>Facebook Help</a>.<br/> ";
        msg += "Did you grant Digester access to your e-mail address? ";
        msg +=
          "If not, please review your settings here: <a style='color: black' href='https://www.facebook.com/help/218345114850283' target='_blank'>Facebook Privacy Settings</a>.<br />";
        msg += "<br />If nothing helps, please contact us at info@digester.app";
        return {
          title: "Looks like we could not access your E-Mail Address",
          message: msg
        };
      } else {
        return null;
      }
    }
  },

  mounted() {
    const isAuthenticated = this.$store.getters.isAuthenticated;
    if (isAuthenticated) {
      this.$router.push("/cockpit");
    }
  }
};
</script>