# RmDb
### Project in Rust with Rocket/SQLx/Tera

The idea of the project is to have a database with movies, and let users submit their reviews, with a score.

Might add more features later...

### Main objectives
- [x] ~~Create DB~~
- [x] ~~Create Home template~~
- [x] ~~Add Movies to DB~~
- [x] ~~Create CRUD for Movies~~
- [x] ~~Add templates for Movies (create/edit/delete)~~
- [x] ~~Add Users to DB~~
- [x] ~~Create auth for Users~~
- [x] ~~Add templates for Users (Sign/login/logout)~~
- [x] ~~Add Reviews to DB (user <--> movie)~~
- [x] ~~Add template to review the movies~~
- [x] ~~Show reviews on the movie template~~
- [x] ~~Show Avg score for the movies~~


### Refactors
- [ ] Better redirects
- [ ] Creation of users
- [ ] Front-end
- [ ] Unwraps

### Instructions to run:

1. Run ```docker compose up```
2. Run migrations ```sqlx migrate run``` (requires ```sqlx-cli```)
3. Create ```.env``` file (use ```.env.example``` for reference)
4. Run project ``` cargo run ```

</br>

<p align="center">
    <img align="center" alt="ferris" width="400" src="https://github.com/Axl-91/rmdb/blob/main/static/images/ferris.png">
</p>
