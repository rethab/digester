<template>
  <v-container>
    <v-row>
      <v-col>
        <h1 class="display-4 d-flex justify-center font-weight-medium">Introducing Digester</h1>
        <p class="mt-5 headline font-italic d-flex justify-center">
          <vue-typed-js
            :strings="['weekly', 'daily']"
            :startDelay="0"
            :backDelay="2500"
            :loop="true"
            :showCursor="false"
            :typeSpeed="200"
            :backSpeed="100"
          >
            <span>
              Get
              <span style="text-decoration: underline" class="typing"></span> digests instead of instant notifications
            </span>
          </vue-typed-js>
        </p>
      </v-col>
    </v-row>

    <v-row style="height: 200px" class="mt-3" align="center" justify="center">
      <v-col cols="3">
        <v-img height="200px" contain src="http://localhost:8080/focus_working.svg"></v-img>
      </v-col>
      <v-col cols="6">
        <h2 class="headline mb-5">Focus On What Matters</h2>
        <p
          class="body-1"
        >Digester allows you to get updates when you want them. Saying no to constant interruptions lets you focus on work during work time.</p>
      </v-col>
    </v-row>

    <v-row style="height: 200px" class="mt-6" align="center" justify="center">
      <v-col cols="6">
        <h2 class="headline mb-5">Spend More Time With Your Family</h2>
        <p
          class="body-1"
        >Say no to FOMO. Digester's ability to bundle updates allows you to spend time with your family while knowing you'll get your updates eventually.</p>
      </v-col>
      <v-col cols="3">
        <v-img height="200px" contain src="http://localhost:8080/family_3.svg"></v-img>
      </v-col>
    </v-row>

    <AuthLogin v-if="!isAuthenticated" />
    <InitializeTimezone v-if="isAuthenticated && firstLogin" />
    <p
      v-else
      class="d-flex justify-center mt-10"
    >You are logged in. Please use the menu on the left.</p>
  </v-container>
</template>

<script>
import AuthLogin from "@/components/AuthLogin.vue";
import InitializeTimezone from "@/components/InitializeTimezone.vue";
export default {
  data() {
    return {
      firstLogin: this.$route.query.firstLogin
    };
  },
  components: {
    AuthLogin,
    InitializeTimezone
  },
  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    }
  }
};
</script>
