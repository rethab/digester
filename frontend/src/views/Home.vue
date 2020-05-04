<template>
  <v-container>
    <v-row justify="center">
      <v-col class="text-center">
        <h1 class="display-2 font-weight-medium">Introducing Digester</h1>
        <p class="title mt-5 font-weight-regular font-italic d-flex justify-center">
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

    <v-row justify="center">
      <v-col class="text-center" sm="6" md="3">
        <v-btn
          @click="$vuetify.goTo('#howitworks')"
          class="mx-4"
          color="secondary"
          outlined
          large
        >how it works</v-btn>
      </v-col>
      <v-col class="text-center" sm="6" md="3">
        <v-btn @click="$vuetify.goTo('#login')" class="mx-4" color="primary" large>try now</v-btn>
        <br />
        <span class="caption font-weight-light">(it's free)</span>
      </v-col>
    </v-row>

    <LandingPageCard
      :img-src="require('@/assets/focus_working.svg')"
      :img-left="true"
      title="What is Digester?"
      content="Create subscriptions for Blogs, Twitter profiles or Github projects and receive weekly (or daily) updates directly in your E-Mail inbox.<br /><br />Fancy reading the NY Times every day at 7am? Start using digester today!"
    />

    <LandingPageCard
      :img-src="require('@/assets/social_dashboard.svg')"
      :img-left="false"
      title="Why use Digester?"
      content="How often do you get a notification on your phone at an inconvenient time? Whether it is a meeting at work or dinner with friends, apps are trying to get your attention.<br /><br />Digester is the opposite: It turns your all-day distractions into consolidated daily or weekly E-Mails. Stay focussed without missing important updates or news!"
    />

    <section id="howitworks">
      <HowItWorksCard />
    </section>

    <section id="login">
      <v-row class="text-center mt-6" justify="center">
        <v-col>
          <h1 class="display-1 font-weight-regular">Try Now</h1>
          <div v-if="!isAuthenticated">
            <AuthLogin />
          </div>
          <div v-else>
            <p class="mt-5">
              Hooray! You are already logged in. Head over to
              <v-btn to="/cockpit" text outlined small class="mr-1" color="secondary">cockpit</v-btn>and start profiting.
            </p>
          </div>
        </v-col>
      </v-row>
    </section>
  </v-container>
</template>

<script>
import AuthLogin from "@/components/auth/AuthLogin.vue";
import LandingPageCard from "@/components/landing/LandingPageCard.vue";
import HowItWorksCard from "@/components/landing/HowItWorksCard.vue";
export default {
  data() {
    return {
      mobile: this.$vuetify.breakpoint.smAndDown
    };
  },
  components: {
    AuthLogin,
    LandingPageCard,
    HowItWorksCard
  },
  computed: {
    isAuthenticated() {
      return this.$store.getters.isAuthenticated;
    }
  }
};
</script>
