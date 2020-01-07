<template>
  <v-container>
    <InitializeTimezone v-if="isAuthenticated && firstLogin" />
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
        <v-btn @click="$vuetify.goTo('#login')" class="mx-4" color="primary" large>Try now for Free</v-btn>
      </v-col>
      <v-col class="text-center" sm="6" md="3">
        <v-btn
          @click="$vuetify.goTo('#howitworks')"
          class="mx-4"
          color="primary"
          outlined
          large
        >how it works</v-btn>
      </v-col>
    </v-row>

    <LandingPageCard
      img-src="img/focus_working.svg"
      :img-left="true"
      title="Focus on What Matters"
      content="Digester allows you to get updates when you want them. Saying no to constant interruptions lets you focus on work during work time."
    />

    <LandingPageCard
      img-src="img/family_3.svg"
      :img-left="false"
      title="Spend More Time With Your Family"
      content="Say no to FOMO. Digester's ability to bundle updates allows you to spend time with your family while knowing you'll get your updates eventually."
    />

    <LandingPageCard
      img-src="img/following_the_idea.svg"
      :img-left="true"
      title="Supported Features"
      content="You can currently subscribe to Github releases via E-Mail. This means you'll get digests for new versions of your favourite projects.<br /> <br />More features are coming soon: We plan to support digests via Slack instead of E-Mail and many more sources such as YouTube, Blogs/RSS and many more."
    />

    <section id="howitworks">
      <HowItWorksCard />
    </section>

    <v-row class="mt-6" align="center" justify="center">
      <v-col cols="9">
        <section id="login">
          <h1 class="display-2 d-flex justify-center font-weight-medium">Try Now For Free</h1>
          <div v-if="!isAuthenticated" class="text-center">
            <p
              class="font-italic mt-5"
            >Click on the button below to sign in and start creating subscriptions.</p>
            <GithubLoginBtn />
          </div>
          <div v-else class="text-center">
            <p class="mt-5">
              Hooray! You are already logged in. Head over to
              <v-btn to="/subs" text outlined small color="primary">subscriptions</v-btn>and start profiting.
            </p>
          </div>
        </section>
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
import GithubLoginBtn from "@/components/GithubLoginBtn.vue";
import LandingPageCard from "@/components/LandingPageCard.vue";
import HowItWorksCard from "@/components/HowItWorksCard.vue";
import InitializeTimezone from "@/components/InitializeTimezone.vue";
export default {
  data() {
    return {
      firstLogin: this.$route.query.firstLogin,
      mobile: this.$vuetify.breakpoint.smAndDown
    };
  },
  components: {
    InitializeTimezone,
    GithubLoginBtn,
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
