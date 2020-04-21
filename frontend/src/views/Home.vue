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
        <v-btn @click="$vuetify.goTo('#howitworks')" class="mx-4" color="primary" large>how it works</v-btn>
      </v-col>
      <v-col class="text-center" sm="6" md="3">
        <v-btn
          @click="$vuetify.goTo('#login')"
          class="mx-4"
          color="secondary"
          outlined
          large
        >try for free</v-btn>
      </v-col>
    </v-row>

    <LandingPageCard
      :img-src="require('@/assets/following_the_idea.svg')"
      :img-left="true"
      title="What is Digester?"
      content="Want to follow a Blog? A Twitter profile? A Github project?<br /><br />Knowing when new posts are published can be cumbersome and spending too much time on Twitter is no good. <br /><br />Digester is here to help: Create subscriptions for a Blogs, Twitter profiles or Github projects and receive weekly (or daily) updates directly in your E-Mail inbox."
    />

    <LandingPageCard
      :img-src="require('@/assets/focus_working.svg')"
      :img-left="false"
      title="Bundle Topics with Lists"
      content="Lists are collections of Blogs, Twitter profiles and Github projects.<br /><br />Say you are intersted in mobile development. By subscribing to the mobile development list, you'll receive a weekly digest on what happened in the world of mobile. A new Blog post about flutter? A new release of Swift? All part of your digest!<br /><br />You can even create your own lists and share them with your friends."
    />

    <section id="howitworks">
      <HowItWorksCard />
    </section>

    <section id="login">
      <v-row class="text-center mt-6" justify="center">
        <v-col cols="9">
          <h1 class="display-1 font-weight-regular">Try Now For Free</h1>
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
