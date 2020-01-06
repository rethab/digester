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

    <v-row>
      <v-col justify="center">
        <div class="text-center">
          <v-btn
            @click="$vuetify.goTo('#login')"
            class="mx-4"
            color="primary"
            large
          >Try now for Free</v-btn>
          <v-btn
            @click="$vuetify.goTo('#howitworks')"
            class="mx-4"
            color="primary"
            outlined
            large
          >how it works</v-btn>
        </div>
      </v-col>
    </v-row>

    <v-row style="height: 200px" class="mt-3" align="center" justify="center">
      <v-col cols="3">
        <v-img height="200px" contain src="focus_working.svg"></v-img>
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
        <v-img height="200px" contain src="family_3.svg"></v-img>
      </v-col>
    </v-row>

    <section id="howitworks">
      <v-row class="mt-6" align="center" justify="center">
        <v-col cols="9">
          <h1 class="display-2 d-flex justify-center font-weight-medium">How It Works</h1>
          <p
            class="mt-3 headline font-italic d-flex justify-center"
          >Create Subscription. Receive Digests. Relax.</p>
          <v-card elevation="24">
            <v-carousel light show-arrows-on-hover cycle>
              <v-carousel-item>
                <v-img contain src="add-subscription.png"></v-img>
              </v-carousel-item>
              <v-carousel-item>
                <v-img contain src="digest-email-cut.png"></v-img>
              </v-carousel-item>
            </v-carousel>
          </v-card>
        </v-col>
      </v-row>
    </section>

    <v-row class="mt-6" align="center" justify="center">
      <v-col cols="9">
        <section id="login">
          <h1 class="display-2 d-flex justify-center font-weight-medium">Try Now For Free</h1>
          <p class="mt-3 headline font-italic d-flex justify-center">Sign in using GitHub</p>
        </section>
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
