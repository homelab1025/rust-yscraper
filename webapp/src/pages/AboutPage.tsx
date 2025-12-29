import { Typography, Container, Paper } from '@mui/material';

const AboutPage = () => {
  return (
    <Container>
      <Paper sx={{ p: 3 }}>
        <Typography variant="h4" gutterBottom>
          About YScraper
        </Typography>
        <Typography variant="body1">
          YScraper is a tool designed to scrape and manage comments from various sources,
          starting with Hacker News. It allows you to track specific discussions by
          adding links and automatically fetching the latest comments for your review.
        </Typography>
      </Paper>
    </Container>
  );
};

export default AboutPage;
