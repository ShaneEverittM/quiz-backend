CREATE TABLE quiz (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(240) NOT NULL,
    description VARCHAR(240) NOT NULL
);
CREATE TABLE question (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    description VARCHAR(240) NOT NULL,
    qz_id INTEGER NOT NULL,
    FOREIGN KEY(qz_id) REFERENCES quiz(id) ON DELETE CASCADE
);
CREATE TABLE answer (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    description VARCHAR(240) NOT NULL,
    val INTEGER NOT NULL,
    q_id INTEGER NOT NULL,
    FOREIGN KEY(q_id) REFERENCES question(id) ON DELETE CASCADE
);
CREATE TABLE result (
    id INTEGER AUTO_INCREMENT PRIMARY KEY,
    num INTEGER NOT NULL,
    header VARCHAR(64) NOT NULL,
    description VARCHAR(240) NOT NULL,
    qz_id INTEGER NOT NULL,
    FOREIGN KEY(qz_id) REFERENCES quiz(id) ON DELETE CASCADE
);
create view topQuizzes as
select quiz.name,
    quiz.id,
    quiz.description
from quiz;