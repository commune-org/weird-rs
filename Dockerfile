FROM postgres
ENV POSTGRES_USER=YOUR_USERNAME
ENV POSTGRES_PASSWORD=YOUR_PASSWORD
ENV POSTGRES_DB=YOUR_DATABASE
EXPOSE 5432
COPY pg-setup.sql /docker-entrypoint-initdb.d
CMD ["postgres"]
